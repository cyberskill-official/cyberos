# Dockerfile - carry the CyberOS payload and install into any repo mounted at /work.
# Build from a built pack dir:  docker build -t cyberos /path/to/dist/cyberos
# Install into a repo:          docker run --rm -v "$PWD":/work cyberos
#
# Note: this image installs the machine. It does NOT carry your project's toolchain, so run
# the gates on a runner that has your build/test tools:
#   docker run --rm -v "$PWD":/work --entrypoint bash cyberos \
#     /opt/cyberos/cuo/gates/run-gates.sh    # only if the toolchain is in the image
FROM alpine:3.20
RUN apk add --no-cache bash git coreutils curl tar
COPY . /opt/cyberos
RUN test -f /opt/cyberos/install.sh || { echo "payload has no install.sh"; exit 1; }
RUN chmod +x /opt/cyberos/install.sh /opt/cyberos/cuo/gates/run-gates.sh 2>/dev/null || true
WORKDIR /work
ENTRYPOINT ["bash","/opt/cyberos/install.sh","/work"]
