#!/bin/sh
# Finish the cyberos-skill-broker clippy fix.
#
# Background: the Cowork sandbox edited the file and verified the fix
# (cargo clippy --all-targets -D warnings + 15/15 tests pass), but the
# mount left a stuck .git/index.lock + .git/HEAD.lock that the sandbox
# couldn't unlink. The commit object was already written to git's object
# store as ea43cd484e2267e1dd6bd1e861c574e9d23375ce.
#
# Pick ONE of the two options below — both leave you in the same state.

set -eu
cd "$(dirname "$0")"

# Clear any stale lockfiles from the sandbox commit attempt.
rm -f .git/index.lock .git/HEAD.lock .git/refs/heads/main.lock

# Option A — fast-forward main to the prebuilt commit (no re-running git):
git update-ref refs/heads/main ea43cd484e2267e1dd6bd1e861c574e9d23375ce
git push

# Option B — recommit normally (if you'd rather see git stage the diff):
# git add services/skill-broker/src/transpilers/anthropic.rs
# git commit -m "fix(skill-broker): clippy clean-up in anthropic transpiler"
# git push
