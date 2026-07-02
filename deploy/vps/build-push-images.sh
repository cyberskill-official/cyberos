#!/usr/bin/env bash
# Build the CyberOS P0 service images for the VPS and push them to GHCR - the LOCAL replacement for the old
# GitHub build job. Run on your Mac.
#
#   one-time:  docker login ghcr.io -u <github-username>      # use a PAT with write:packages
#   build:     bash deploy/vps/build-push-images.sh           # auth + chat (eval is opt-in)
#
# The VPS is x86_64, so images are built for linux/amd64. On an Apple Silicon Mac that runs under emulation
# and is slower than a native build - that is the trade for keeping the build off GitHub. Each image gets
# :latest plus a second tag (the short git sha by default; override with IMAGE_TAG).
#
# After this finishes, deploy: trigger the "deploy" workflow (GitHub Actions -> deploy -> Run workflow), or
# run `bash ~/cyberos/deploy/vps/deploy.sh` on the VPS. Either one git-pulls the console/Caddyfile/compose
# and docker-pulls these images.
set -euo pipefail

OWNER="${GHCR_OWNER:-cyberskill-official}"
REGISTRY="ghcr.io/${OWNER}"
TAG="${IMAGE_TAG:-$(git rev-parse --short HEAD 2>/dev/null || echo local)}"
PLATFORM="${PLATFORM:-linux/amd64}"

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT/services"

# A buildx builder that can emit linux/amd64 and push a real manifest. Reuse it if it already exists.
if ! docker buildx inspect cyberos-builder >/dev/null 2>&1; then
  docker buildx create --name cyberos-builder --driver docker-container --use >/dev/null
fi
docker buildx use cyberos-builder

build_push() {
  local pkg="$1"
  local bin="${2:-$1}"
  echo "==> building ${pkg} (${PLATFORM}) -> ${REGISTRY}/${pkg}:latest + :${TAG}"
  docker buildx build \
    --platform "${PLATFORM}" \
    --file Dockerfile \
    --build-arg "PACKAGE=${pkg}" \
    --build-arg "BIN=${bin}" \
    --tag "${REGISTRY}/${pkg}:latest" \
    --tag "${REGISTRY}/${pkg}:${TAG}" \
    --push \
    .
}

build_push cyberos-auth
build_push cyberos-chat
# The gateway package and its server bin have different names (cyberos-ai-gateway / cyberos-gateway).
build_push cyberos-ai-gateway cyberos-gateway

# The bge-m3 embed sidecar is Python with its own Dockerfile and build context (services/embed-sidecar).
echo "==> building cyberos-embed-sidecar (${PLATFORM}) -> ${REGISTRY}/cyberos-embed-sidecar:latest + :${TAG}"
docker buildx build \
  --platform "${PLATFORM}" \
  --file embed-sidecar/Dockerfile \
  --tag "${REGISTRY}/cyberos-embed-sidecar:latest" \
  --tag "${REGISTRY}/cyberos-embed-sidecar:${TAG}" \
  --push \
  embed-sidecar

# eval (BRAIN/EVAL) is still stabilising and is OFF by default in the deploy, so it is not built unless asked.
if [ "${BUILD_EVAL:-0}" = "1" ]; then
  build_push cyberos-eval
else
  echo "==> skipping eval (set BUILD_EVAL=1 to build + push it)"
fi

echo "==> images pushed. Deploy with: Actions -> deploy -> Run workflow (or deploy.sh on the VPS)."
