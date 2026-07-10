# services/ developer shortcuts. Run from this directory.

.PHONY: fmt clippy build test ci-mirror

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
