# services/ developer shortcuts. Run from this directory.

.PHONY: fmt clippy build test ci-mirror chat-build chat-license-check chat-test chat-verify

# Auto-format every crate in the workspace.
fmt:
	cargo fmt --all

# Strict lint. CI runs this as a gate.
clippy:
	cargo clippy --workspace --all-targets --no-deps -- -D warnings

# Whole-workspace build (no tests).
build:
	cargo build --workspace

# Pure-Rust tests (skips #[ignore]'d Postgres-required tests).
test:
	cargo test --workspace --no-fail-fast

# Mirrors what GitHub Actions runs in the lint-and-test job.
# Run this before pushing to catch issues locally.
ci-mirror: fmt clippy build test
	@echo "✓ ci-mirror green — push when ready"

# ----------------------------------------------------------------------------
# FR-CHAT-001 — CHAT module shortcuts (delegate to services/chat/Makefile).
# ----------------------------------------------------------------------------

# Build the Mattermost fork image from the pinned commit + patch series.
chat-build:
	$(MAKE) -C chat chat-build

# Run the license-drift watcher locally (requires GH_TOKEN).
chat-license-check:
	$(MAKE) -C chat chat-license-check

# Run the FR-CHAT-001 bash-level test suite.
chat-test:
	$(MAKE) -C chat chat-test

# Verify the layout invariants of services/chat/ (no Docker needed).
chat-verify:
	$(MAKE) -C chat chat-verify
