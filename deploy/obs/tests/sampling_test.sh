#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

cd "$root/services"

cargo test -p cyberos-obs-collector tail_sampling -- --nocapture
cargo test -p cyberos-obs-collector config::tests::validate_accepts_repo_configs -- --nocapture

echo "sampling_test passed"
