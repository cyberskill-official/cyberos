"""FastAPI HTTP server serving the embedding endpoint.

Two wire shapes over the same model:

  GET  /healthz
  POST /embed            — native shape, wire-compatible with
       `services/memory/src/embeddings.rs`:
       { "texts": [...], "model": "bge-m3" }
       → { "embeddings": [[float; 1024]; N], "model": "bge-m3", "dim": 1024 }
  POST /v1/embeddings    — OpenAI-compatible shape, wire-compatible with the
       ai-gateway's `local_openai` provider (`LocalOpenaiProvider::call_embed`
       posts `{model, input}` and parses `{data: [{embedding}], usage}`):
       { "model": "bge-m3", "input": "one" | ["many", ...] }
       → { "object": "list", "data": [{ "object": "embedding", "index": i,
           "embedding": [..1024..] }], "model": "bge-m3",
           "usage": { "prompt_tokens": n, "total_tokens": n } }

Boot warmup (prod): real mode lazy-loads ~2.3 GB of weights on first use, so a
cold first call can exceed a caller's deadline. Set `CYBEROS_EMBED_WARMUP=1`
to start loading in a background thread at startup; `/healthz` reports the
warmup state (`warm`) so orchestration can wait for `ready` before routing
traffic.
"""

from __future__ import annotations

import contextlib
import hashlib
import math
import os
import struct
import threading
from typing import Literal

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field

EMBED_DIM = 1024
MODEL_NAME = "bge-m3"

# Warmup state machine: cold → warming → ready | failed:<reason>. Mock mode is
# always "ready" (there is nothing to load). Written only by the warmup thread
# and the real-backend loader; read by /healthz.
_warm_state = "cold"
_warm_lock = threading.Lock()


def _set_warm(state: str) -> None:
    global _warm_state
    with _warm_lock:
        _warm_state = state


def _get_warm() -> str:
    if os.environ.get("CYBEROS_EMBED_MODE", "real").lower() == "mock":
        return "ready"
    with _warm_lock:
        return _warm_state


def _warmup_thread() -> None:
    """Load the real backend off the request path. Any failure is recorded by
    the loader itself, never raised — the service stays up and the first real
    request retries."""
    try:
        _get_real_backend()
    except Exception:  # noqa: BLE001 — state already recorded by the loader
        pass


@contextlib.asynccontextmanager
async def _lifespan(app: FastAPI):
    if (
        os.environ.get("CYBEROS_EMBED_WARMUP", "0") == "1"
        and os.environ.get("CYBEROS_EMBED_MODE", "real").lower() == "real"
    ):
        threading.Thread(target=_warmup_thread, name="embed-warmup", daemon=True).start()
    yield


app = FastAPI(title="cyberos-embed-sidecar", version="0.1.0", lifespan=_lifespan)


class EmbedRequest(BaseModel):
    texts: list[str] = Field(..., min_length=1)
    model: str = MODEL_NAME


class EmbedResponse(BaseModel):
    embeddings: list[list[float]]
    model: str
    dim: int


# ---------------------------------------------------------------------------
# Backend resolution — lazy. We don't load the 2.3 GB model unless we have
# to, and tests can override via the env var.
# ---------------------------------------------------------------------------

_real_backend = None

# Serialises model loading so a warmup thread and a first request can't load
# the 2.3 GB weights twice concurrently (a 2x RAM spike the small VPS cannot
# absorb).
_load_lock = threading.Lock()


def _mock_embed(text: str) -> list[float]:
    """Deterministic 1024-dim unit-norm vector keyed by SHA-256(text).

    The vector is meaningful (similar inputs produce dissimilar outputs —
    by design, since we hash the WHOLE string), but consistent across runs
    so tests on the Rust side can assert byte-stable behaviour.
    """
    digest = hashlib.sha256(text.encode("utf-8")).digest()
    # Stretch the 32-byte digest to 1024 floats by chunking into 4-byte
    # groups + interpreting as IEEE 754 + tiling.
    pieces: list[float] = []
    seed_floats = []
    for i in range(0, 32, 4):
        f = struct.unpack("!f", digest[i:i + 4])[0]
        if not math.isfinite(f):
            f = 0.0
        seed_floats.append(f)
    # Tile 8 floats → 1024 by repeating + adding a small step so adjacent
    # dimensions aren't identical (helps test reranking).
    for i in range(EMBED_DIM):
        base = seed_floats[i % 8]
        step = (i // 8) * 1e-5
        pieces.append(base + step)
    # Unit-norm so cosine similarity is well-defined.
    norm = math.sqrt(sum(x * x for x in pieces)) or 1.0
    return [x / norm for x in pieces]


def _get_real_backend():
    """Lazy-load the sentence-transformers model (thread-safe, load-once).
    Errors visibly if the `real` extras aren't installed, and records the
    warmup state either way."""
    global _real_backend
    with _load_lock:
        if _real_backend is not None:
            return _real_backend
        _set_warm("warming")
        try:
            from sentence_transformers import SentenceTransformer
        except ImportError as e:
            _set_warm(f"failed: real extras not installed ({e})")
            raise HTTPException(
                status_code=503,
                detail=(
                    "real backend requires the `real` extras: "
                    "`pip install 'cyberos-embed-sidecar[real]'`. "
                    f"underlying ImportError: {e}"
                ),
            )
        device = os.environ.get("CYBEROS_EMBED_DEVICE", "cpu")
        try:
            _real_backend = SentenceTransformer("BAAI/bge-m3", device=device)
        except Exception as e:
            _set_warm(f"failed: {e}")
            raise
        _set_warm("ready")
        return _real_backend


def _real_embed(texts: list[str]) -> list[list[float]]:
    model = _get_real_backend()
    vectors = model.encode(texts, normalize_embeddings=True)
    out: list[list[float]] = []
    for v in vectors:
        # SentenceTransformer returns numpy; convert defensively without a
        # hard dep on numpy in this module's signature.
        try:
            out.append([float(x) for x in v.tolist()])
        except AttributeError:
            out.append([float(x) for x in v])
    return out


# ---------------------------------------------------------------------------
# Public surface — selects backend per call so an env-var flip takes effect
# without restart.
# ---------------------------------------------------------------------------

def embed_texts(texts: list[str], mode: str | None = None) -> list[list[float]]:
    mode = (mode or os.environ.get("CYBEROS_EMBED_MODE", "real")).lower()
    if mode == "mock":
        return [_mock_embed(t) for t in texts]
    if mode == "real":
        return _real_embed(texts)
    raise HTTPException(status_code=500, detail=f"unknown CYBEROS_EMBED_MODE: {mode!r}")


@app.get("/healthz")
def healthz() -> dict:
    return {
        "service": "cyberos-embed-sidecar",
        "model": MODEL_NAME,
        "dim": EMBED_DIM,
        "mode": os.environ.get("CYBEROS_EMBED_MODE", "real"),
        "warm": _get_warm(),
    }


@app.post("/embed", response_model=EmbedResponse)
def embed(req: EmbedRequest) -> EmbedResponse:
    if req.model and req.model != MODEL_NAME:
        raise HTTPException(
            status_code=400,
            detail=f"unsupported model: {req.model!r}; this sidecar serves {MODEL_NAME!r}",
        )
    vectors = embed_texts(req.texts)
    return EmbedResponse(embeddings=vectors, model=MODEL_NAME, dim=EMBED_DIM)


# ---------------------------------------------------------------------------
# OpenAI-compatible surface — what the ai-gateway's `local_openai` provider
# speaks (DEC-2723 wiring: gateway `embed.standard` → this sidecar).
# ---------------------------------------------------------------------------


class OpenAiEmbedRequest(BaseModel):
    """`POST /v1/embeddings` body. `input` accepts the OpenAI union of one
    string or a list of strings; a bare string is treated as a batch of one."""

    input: str | list[str]
    model: str = MODEL_NAME


class OpenAiEmbedItem(BaseModel):
    object: Literal["embedding"] = "embedding"
    index: int
    embedding: list[float]


class OpenAiUsage(BaseModel):
    prompt_tokens: int
    total_tokens: int


class OpenAiEmbedResponse(BaseModel):
    object: Literal["list"] = "list"
    data: list[OpenAiEmbedItem]
    model: str
    usage: OpenAiUsage


@app.post("/v1/embeddings", response_model=OpenAiEmbedResponse)
def openai_embeddings(req: OpenAiEmbedRequest) -> OpenAiEmbedResponse:
    if req.model and req.model != MODEL_NAME:
        raise HTTPException(
            status_code=400,
            detail=f"unsupported model: {req.model!r}; this sidecar serves {MODEL_NAME!r}",
        )
    texts = [req.input] if isinstance(req.input, str) else req.input
    if not texts:
        raise HTTPException(status_code=400, detail="input must not be empty")
    vectors = embed_texts(texts)
    # Whitespace token count — an estimate, present because the OpenAI shape
    # carries usage and the gateway reads prompt_tokens. Local inference is
    # zero-cost, so precision does not matter here.
    tokens = sum(len(t.split()) for t in texts)
    return OpenAiEmbedResponse(
        data=[OpenAiEmbedItem(index=i, embedding=v) for i, v in enumerate(vectors)],
        model=MODEL_NAME,
        usage=OpenAiUsage(prompt_tokens=tokens, total_tokens=tokens),
    )


def run() -> None:
    """Entry point for the `cyberos-embed-sidecar` console script."""
    import uvicorn

    host = os.environ.get("CYBEROS_EMBED_HOST", "127.0.0.1")
    port = int(os.environ.get("CYBEROS_EMBED_PORT", "7900"))
    uvicorn.run(app, host=host, port=port)
