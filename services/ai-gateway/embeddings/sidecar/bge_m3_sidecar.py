"""FastAPI BGE-M3 embedding sidecar."""

from __future__ import annotations

import os
import time
from pathlib import Path

import torch
from fastapi import FastAPI, HTTPException
from sentence_transformers import SentenceTransformer

try:
    from .batch import EmbedRequest, EmbedResponse, MAX_TOKENS, apply_task_prefix, rough_token_count
    from .checksum import ChecksumMismatch, verify_model_checksum
    from .health import HealthState
except ImportError:  # pragma: no cover - supports direct uvicorn module loading
    from batch import EmbedRequest, EmbedResponse, MAX_TOKENS, apply_task_prefix, rough_token_count
    from checksum import ChecksumMismatch, verify_model_checksum
    from health import HealthState

SIDECAR_VERSION = "1.0.0"
MODEL_NAME = "bge-m3"

app = FastAPI(title="CyberOS BGE-M3 Sidecar", version=SIDECAR_VERSION)
health = HealthState(SIDECAR_VERSION)


def _model_path() -> Path:
    return Path(os.environ.get("BGE_M3_MODEL_PATH", "/models/bge-m3"))


def _checksum_path() -> Path:
    return Path(os.environ.get("BGE_M3_CHECKSUM_PATH", "/app/checksums/bge-m3.sha256"))


def _select_device() -> str:
    return "cuda" if torch.cuda.is_available() else "cpu"


@app.on_event("startup")
async def startup() -> None:
    try:
        full_sha = verify_model_checksum(_model_path(), _checksum_path())
        device = _select_device()
        backend = "onnx" if device == "cpu" else None
        if backend:
            app.state.model = SentenceTransformer(str(_model_path()), device=device, backend=backend)
        else:
            app.state.model = SentenceTransformer(str(_model_path()), device=device)
        app.state.model_sha256 = full_sha[:16]
        app.state.device = device
        health.set_ready(device=device, model_sha256=full_sha)
    except Exception as exc:
        health.set_error(exc)
        if isinstance(exc, ChecksumMismatch):
            raise
        raise


@app.get("/health")
async def healthcheck() -> dict[str, str | None]:
    snapshot = health.snapshot()
    if snapshot.status != "ok":
        raise HTTPException(status_code=503, detail=snapshot.error or snapshot.status)
    return {
        "status": snapshot.status,
        "device": snapshot.device,
        "sidecar_version": snapshot.sidecar_version,
        "model_sha256": snapshot.model_sha256,
    }


@app.post("/embed", response_model=EmbedResponse)
async def embed(req: EmbedRequest) -> EmbedResponse:
    snapshot = health.snapshot()
    if snapshot.status != "ok":
        raise HTTPException(status_code=503, detail=snapshot.error or snapshot.status)

    prepared: list[str] = []
    for idx, text in enumerate(req.texts):
        actual = rough_token_count(text)
        if actual > MAX_TOKENS:
            raise HTTPException(
                status_code=413,
                detail={
                    "error": "input_too_long",
                    "max_tokens": MAX_TOKENS,
                    "actual_tokens": actual,
                    "text_index": idx,
                },
            )
        prepared.append(apply_task_prefix(text, req.task))

    started = time.perf_counter()
    device = _select_device()
    embeddings = app.state.model.encode(prepared, normalize_embeddings=True).tolist()
    elapsed_ms = int((time.perf_counter() - started) * 1000)
    return EmbedResponse(
        embeddings=embeddings,
        model_name=MODEL_NAME,
        model_sha256=app.state.model_sha256,
        sidecar_version=SIDECAR_VERSION,
        device=device,
        elapsed_ms=elapsed_ms,
    )
