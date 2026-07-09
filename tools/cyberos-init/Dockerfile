# Dockerfile - carry the CyberOS payload and scaffold any repo mounted at /work.
# Build from a built pack dir:  docker build -t cyberos /path/to/dist/cyberos
# Init a repo:                  docker run --rm -v "$PWD":/work cyberos
#
# Note: this image scaffolds (init) the repo. It does NOT carry your project's
# toolchain, so run the gates on a runner that has your build/test tools:
#   docker run --rm -v "$PWD":/work --entrypoint bash cyberos \
#     /opt/cyberos/gates/run-gates.sh    # only if the toolchain is in the image
FROM alpine:3.20
RUN apk add --no-cache bash git coreutils curl tar
COPY . /opt/cyberos
RUN chmod +x /opt/cyberos/init.sh /opt/cyberos/gates/run-gates.sh 2>/dev/null || true
WORKDIR /work
ENTRYPOINT ["bash","/opt/cyberos/init.sh","/work"]
