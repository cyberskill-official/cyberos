---
description: Show the installed CyberOS version, rules fingerprint, and changelog pointers. Manual only — not auto-run.
argument-hint: "[repo path, default: current repo]"
---
Report CyberOS install status for repo_root = ${1:-the current repo}:

1. Run `bash .cyberos/status.sh` when present, else read `.cyberos/VERSION` + `.cyberos/manifest.yaml` (version, rules_sha, built_at).

2. If a payload is reachable, note whether `update` would report stale.

3. Point at the published changelog: https://github.com/cyberskill-official/cyberos/blob/main/CHANGELOG.md

Keep it short and factual. This is a manual report only (not part of the soft auto-check path).
