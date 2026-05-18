"""FastAPI HTTP server serving the embedding endpoint.

Wire-compatible with `services/brain/src/embeddings.rs`. Endpoints:

  GET  /healthz
  POST /embed   { "texts": [...], "model": "bge-m3" }
       → { "embeddings": [[float; 1024]; N], "model": "bge-m3", "dim": 1024 }
"""

from __future__ import annotations

import hashlib
import math
import os
import struct
from typing import Literal

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field

EMBED_DIM = 1024
MODEL_NAME = "bge-m3"

app = FastAPI(title="cyberos-embed-sidecar", version="0.1.0")


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
    """Lazy-load the sentence-transformers model. Errors visibly if the
    `real` extras aren't installed."""
    global _real_backend
    if _real_backend is not None:
        return _real_backend
    try:
        from sentence_transformers import SentenceTransformer
    except ImportError as e:
        raise HTTPException(
            status_code=503,
            detail=(
                "real backend requires the `real` extras: "
                "`pip install 'cyberos-embed-sidecar[real]'`. "
                f"underlying ImportError: {e}"
            ),
        )
    device = os.environ.get("CYBEROS_EMBED_DEVICE", "cpu")
    _real_backend = SentenceTransformer("BAAI/bge-m3", device=device)
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


def run() -> None:
    """Entry point for the `cyberos-embed-sidecar` console script."""
    import uvicorn

    host = os.environ.get("CYBEROS_EMBED_HOST", "127.0.0.1")
    port = int(os.environ.get("CYBEROS_EMBED_PORT", "7900"))
    uvicorn.run(app, host=host, port=port)
