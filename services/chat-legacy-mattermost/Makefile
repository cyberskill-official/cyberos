# TASK-CHAT-001 — Makefile for the CHAT service (Mattermost fork).
#
# Targets:
#   chat-build           Build the Docker image at the pinned commit + patch series.
#   chat-license-check   Run the license-drift watcher locally (requires GH_TOKEN).
#   chat-compose-up      Start the local-dev compose stack.
#   chat-compose-down    Tear it down.
#   chat-test            Run the bash-level tests under tests/ and plugin shape tests.
#   chat-verify          Smoke-verify the layout invariants (PINNED_COMMIT shape,
#                        scripts executable, workflows present).

SHELL := /bin/bash

PINNED_FILE := PINNED_COMMIT
PATCH_VERSION_FILE := CYBEROS_PATCH_VERSION

PINNED := $(shell grep -E '^[0-9a-f]{40}' PINNED_COMMIT | head -1 | cut -d' ' -f1)
PATCH_VERSION := $(shell tr -d '[:space:]' < CYBEROS_PATCH_VERSION | head -c 32)

IMAGE_TAG := cyberos/chat:$(shell echo $(PINNED) | cut -c1-12)-$(PATCH_VERSION)

.PHONY: chat-build chat-license-check chat-compose-up chat-compose-down chat-test chat-authbridge-test chat-verify

chat-build:
	@echo "Building $(IMAGE_TAG) from pinned commit $(PINNED) + patch series $(PATCH_VERSION)..."
	docker build \
		--build-arg PINNED_COMMIT=$(PINNED) \
		--build-arg CYBEROS_PATCH_VERSION=$(PATCH_VERSION) \
		-t $(IMAGE_TAG) \
		.
	@echo "✓ Built $(IMAGE_TAG)"

chat-license-check:
	@if [ -z "$$GH_TOKEN" ] && [ -z "$$GITHUB_TOKEN" ]; then \
		echo "::error::GH_TOKEN or GITHUB_TOKEN must be set for chat-license-check" >&2; \
		exit 2; \
	fi
	bash scripts/check-license-drift.sh "$(SINCE_WINDOW)"

chat-compose-up:
	docker compose up -d
	@echo "✓ Local CHAT stack running on http://localhost:8065"

chat-compose-down:
	docker compose down -v

chat-test:
	@bash tests/run_all_tests.sh

chat-authbridge-test:
	@bash plugins/cyberos-authbridge/tests/authbridge_shape_test.sh

chat-verify:
	@bash tests/pinned_commit_test.sh
	@bash tests/patch_apply_test.sh
	@bash tests/workflows_present_test.sh
	@bash plugins/cyberos-authbridge/tests/authbridge_shape_test.sh
	@echo "✓ All TASK-CHAT-001 layout invariants pass"
