---
description: Open the repo's CyberOS status page (docs/status/index.html) in the default browser. That is all this command does.
argument-hint: "[repo path, default: current repo]"
---
Open the status page for repo_root = ${1:-the current repo}.

1. Run `bash .cyberos/status.sh` (or `bash <payload>/status.sh <repo_root>`). That opens `docs/status/index.html` in the default browser.

2. If the page is missing, report that install may not have produced FRs yet, or suggest re-running install / waiting for the first FR.

3. Do not print long version reports here — that is `/version`. Do not re-vendor — that is `/install`.
