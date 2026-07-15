---
description: Remove the vendored CyberOS machine from a repo (keeps tasks and BRAIN by default).
argument-hint: "[repo path, default: current repo]"
---
Uninstall CyberOS from repo_root = ${1:-the current repo}.

1. Confirm with the user first (destructive to the vendored machine under `.cyberos/`).

2. Run `bash .cyberos/uninstall.sh` or `bash <payload>/uninstall.sh <repo_root>`.

3. Defaults: keeps `docs/tasks/`, `docs/status/`, agent pointer files, and the BRAIN store (`.cyberos/memory/store/`). Drop the BRAIN only if the user sets `CYBEROS_UNINSTALL_KEEP_BRAIN=0`.

4. To reinstall later: `/install` or `bash <payload>/install.sh <repo_root>`.
