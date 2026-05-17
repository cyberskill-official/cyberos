---
template: release-notes@1
title: <release-id> release notes
release_id: <SemVer or tag>
release_date: 2026-MM-DD
prior_release_id: <previous tag>
audience: customer_public    # customer_public | customer_enterprise | internal_only | partner
provenance: { source_path: ./CHANGELOG.md, source_hash: sha256:<hash> }
breaking: false
# compat_target_version: <min prior version that can upgrade directly — required if breaking: true>
# security_advisories:
#   - { cve_id: "CVE-YYYY-NNNN", severity: high, summary: "...", mitigation: "..." }
---

# <release-id> release notes

## Highlights

- 2-5 bullets — what most customers care about.

## Added
- <New feature>

## Changed
- <Non-breaking change>

## Deprecated
- <About-to-be-removed>

## Removed
- <Now removed>

## Fixed
- <Bug fix>

## Security
- <Security fix; cite CVE if applicable>

## Upgrade Notes
<!-- Required when breaking: true -->

## Migration Guide
<!-- Required when breaking: true; code examples preferred -->

## Known Issues
- <Issue + workaround>

<!-- ## Acknowledgements           — when audience: customer_public -->
<!-- ## Compliance Notes           — when audience: customer_enterprise -->
<!-- ## AI Model Update Notes      — when release contains an AI-model update -->
