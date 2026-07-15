---
description: Check whether a newer CyberOS is available for this repo. If stale, ask the user; on yes run install (re-vendor). Soft checks already run on any .cyberos use.
argument-hint: "[repo path, default: current repo]"
---
Check CyberOS version for repo_root = ${1:-the current repo}.

1. Soft update-checks already fire on gates, MCP, status-page hooks, help, and status. This is the **manual** check only.

2. Prefer shell: `bash .cyberos/version.sh` or `bash <payload>/version.sh <repo_root>` (uses `check-latest.sh` for published latest). Report lines verbatim:
   - `installed=`
   - `payload=`
   - `latest=`
   - `installed_rules_sha=`
   - `payload_rules_sha=`
   - `verdict=` (`up_to_date` | `repo_stale` | `payload_stale` | `not_installed` | `rules_drift`)

3. On `verdict=repo_stale` or `not_installed`: ask the user if they want to update/install now. On **yes only**, run `bash <payload>/install.sh <repo_root>` (install is the only re-vendor path — never invent a second apply command).

4. On `verdict=payload_stale`: fetch latest from https://github.com/cyberskill-official/cyberos/releases first, then `install.sh`.

5. On `verdict=rules_drift`: the versions match but `installed_rules_sha` ≠ `payload_rules_sha` — the vendored `.cyberos` is running **different rules than this payload ships**, so VERSION alone is a false all-clear. `installed_rules_sha=none` means the install predates the fingerprint entirely. Treat it exactly like `repo_stale`: ask, then `install.sh` on yes. NEVER report a `rules_drift` repo as up to date.

6. When the remote check was skipped (`latest=unknown`), NEVER claim "up to date" from the local-payload comparison alone.

7. Never re-vendor without an explicit user yes.
