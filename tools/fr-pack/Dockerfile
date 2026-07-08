# Dockerfile - carry the FR pack and scaffold any repo mounted at /work.
# Build from a built pack dir:  docker build -t cyberos-fr /path/to/dist/fr-pack
# Init a repo:                  docker run --rm -v "$PWD":/work cyberos-fr
#
# Note: this image scaffolds (init) the repo. It does NOT carry your project's
# toolchain, so run the gates on a runner that has your build/test tools:
#   docker run --rm -v "$PWD":/work --entrypoint bash cyberos-fr \
#     /opt/fr-pack/gates/run-gates.sh    # only if the toolchain is in the image
FROM alpine:3.20
RUN apk add --no-cache bash git coreutils curl tar
COPY . /opt/fr-pack
RUN chmod +x /opt/fr-pack/init.sh /opt/fr-pack/gates/run-gates.sh 2>/dev/null || true
WORKDIR /work
ENTRYPOINT ["bash","/opt/fr-pack/init.sh","/work"]
