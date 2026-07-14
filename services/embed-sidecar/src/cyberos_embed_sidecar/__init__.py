"""TASK-AI-019 — bge-m3 embedding HTTP sidecar.

Two backends:
  * ``"real"`` — sentence-transformers bge-m3 model loaded into memory at
    boot. CPU-only by default; set ``CYBEROS_EMBED_DEVICE=cuda`` to use a GPU.
  * ``"mock"`` — deterministic stub used by tests + dev mode. Hashes the
    input + emits a 1024-float vector. Useful when the model weights aren't
    available (saves ~2.3 GB on dev workstations) or when running offline.

Backend selection is via ``CYBEROS_EMBED_MODE`` env var (default ``real``).
The HTTP API surface is identical between the two backends so the Rust
client doesn't care which one runs.
"""

from .server import EMBED_DIM, app, embed_texts, run

__all__ = ["EMBED_DIM", "app", "embed_texts", "run"]
__version__ = "0.1.0"
