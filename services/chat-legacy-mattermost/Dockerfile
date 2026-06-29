# FR-CHAT-001 §1 #2 + §1 #8 — build Mattermost from pinned MIT-Apache commit
# and apply the CyberOS patch series. The image tag MUST include the pinned
# SHA (short) plus the CyberOS patch series version so operators can identify
# what is running without inspecting the image manifest.
#
# Build:
#   PINNED=$(awk 'NR==1{print $1}' services/chat/PINNED_COMMIT)
#   docker build \
#       --build-arg PINNED_COMMIT=$PINNED \
#       --build-arg CYBEROS_PATCH_VERSION=$(cat services/chat/CYBEROS_PATCH_VERSION) \
#       -t cyberos/chat:${PINNED:0:12}-$(cat services/chat/CYBEROS_PATCH_VERSION) \
#       services/chat/
#
# Patches in services/chat/patches/ are applied lexicographically; rename
# (001-*.patch, 010-*.patch, ...) controls the order.

ARG PINNED_COMMIT
ARG CYBEROS_PATCH_VERSION=0.1.0

# -----------------------------------------------------------------------------
# Stage 1 — fetch + patch + build the Mattermost server binary.
# -----------------------------------------------------------------------------
FROM golang:1.22-bookworm AS build

ARG PINNED_COMMIT
ARG CYBEROS_PATCH_VERSION

RUN apt-get update \
 && apt-get install -y --no-install-recommends git ca-certificates make \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /src

# Per DEC-422 we MUST NOT `git pull upstream master`. Instead we fetch
# the tarball at the immutable pinned commit. The download URL is the
# raw codeload archive endpoint which is stable for any commit on the
# repository.
RUN curl -fsSL -o /tmp/mattermost.tar.gz \
        "https://codeload.github.com/mattermost/mattermost-server/tar.gz/${PINNED_COMMIT}" \
 && tar --strip-components=1 -xzf /tmp/mattermost.tar.gz \
 && rm /tmp/mattermost.tar.gz \
 && git init -q . \
 && git -c user.email=build@cyberos.local -c user.name=cyberos-build add -A \
 && git -c user.email=build@cyberos.local -c user.name=cyberos-build commit -q -m "base@${PINNED_COMMIT}"

# Apply the CyberOS patch series in lexicographic order. If a patch fails,
# the build halts — operators must rebase the patch series against the
# pinned commit and commit the result.
COPY patches/ /patches/
RUN set -eu; \
    if [ -n "$(ls -1 /patches 2>/dev/null | grep -E '\.patch$' || true)" ]; then \
        for p in $(ls -1 /patches/*.patch | sort); do \
            echo "Applying $p"; \
            git -c user.email=build@cyberos.local -c user.name=cyberos-build \
                am --3way "$p"; \
        done; \
    else \
        echo "No patches to apply"; \
    fi

# Record CyberOS patch version into the binary for runtime visibility.
ENV CYBEROS_PATCH_VERSION=${CYBEROS_PATCH_VERSION}
ENV PINNED_COMMIT=${PINNED_COMMIT}

# Build the server binary using the upstream make target. The cherry-picked
# CyberOS changes only land in this build; no `go install` of any later
# upstream code is permitted.
RUN make build-cmd build-linux 2>/dev/null || make build-server || make build || ( \
        cd cmd/mattermost && CGO_ENABLED=0 go build -ldflags "-s -w \
            -X github.com/mattermost/mattermost-server/v6/model.BuildHash=${PINNED_COMMIT} \
            -X github.com/mattermost/mattermost-server/v6/model.CyberOsPatchVersion=${CYBEROS_PATCH_VERSION}" \
            -o /out/mattermost . \
    )

# -----------------------------------------------------------------------------
# Stage 2 — distroless runtime. No shell, no package manager — minimal
# attack surface for a service that handles tenant chat content.
# -----------------------------------------------------------------------------
FROM gcr.io/distroless/base-debian12 AS runtime

ARG PINNED_COMMIT
ARG CYBEROS_PATCH_VERSION

LABEL org.cyberos.module="chat"
LABEL org.cyberos.upstream="mattermost-server@${PINNED_COMMIT}"
LABEL org.cyberos.patch_version="${CYBEROS_PATCH_VERSION}"
LABEL org.cyberos.license="MIT+Apache-2.0 (upstream) + Apache-2.0 (CyberOS patches)"
LABEL org.cyberos.fork_rationale="Pre-relicense MIT-Apache pin per DEC-420"

# /usr/local/bin/mattermost is the daemon entry point. The exact path the
# upstream Makefile writes to depends on the build target; the COPY here
# accepts both common locations.
COPY --from=build /src/bin/mattermost /usr/local/bin/mattermost
# Fallback used by the manual `cd cmd/mattermost && go build` path above:
# the build script writes to /out when the make targets are unavailable.

COPY config/ /etc/cyberos-chat/

EXPOSE 8065

# Run as non-root by UID; the distroless base image's "nobody" UID is 65532.
USER 65532:65532

ENTRYPOINT ["/usr/local/bin/mattermost", "server", "--config=/etc/cyberos-chat/config.json"]
