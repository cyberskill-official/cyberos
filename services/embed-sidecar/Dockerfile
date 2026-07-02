# cyberos-embed-sidecar — bge-m3 embeddings for the ai-gateway (and later the memory service).
#
# CPU-only by design: the VPS has no GPU, and torch's default wheel drags in CUDA. The torch CPU wheel is
# installed first from the PyTorch CPU index so it lands in its own cache layer (code edits don't re-download
# ~200 MB). Model weights are NOT baked into the image: they download from Hugging Face on first load into
# HF_HOME, which the compose file mounts as a named volume — a ~2.3 GB one-time download that survives
# restarts and redeploys. CYBEROS_EMBED_WARMUP=1 starts that load at boot in a background thread, so
# /healthz is immediately live and reports warm: cold|warming|ready|failed.
#
# Build (from services/embed-sidecar):  docker build -t cyberos-embed-sidecar .
# The python:slim base has no curl; health checks use python's urllib (see the compose healthcheck).

FROM python:3.12-slim

# Torch CPU wheel in its own layer (the big, stable dependency).
RUN pip install --no-cache-dir --index-url https://download.pytorch.org/whl/cpu torch

WORKDIR /app
COPY pyproject.toml ./
COPY src ./src
RUN pip install --no-cache-dir ".[real]"

# Non-root; /data holds the Hugging Face cache (mounted as a volume in prod).
RUN useradd --uid 10001 --create-home cyberos \
    && mkdir -p /data/hf \
    && chown -R cyberos:cyberos /data
USER cyberos

ENV HF_HOME=/data/hf \
    CYBEROS_EMBED_MODE=real \
    CYBEROS_EMBED_WARMUP=1 \
    CYBEROS_EMBED_HOST=0.0.0.0 \
    CYBEROS_EMBED_PORT=7900

EXPOSE 7900
CMD ["cyberos-embed-sidecar"]
