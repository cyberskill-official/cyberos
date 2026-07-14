---
id: NFR-DOC-009
title: "DOC third-party import validation — imported signed docs MUST be signature-verified"
module: DOC
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of imported signed docs verified at import; invalid signatures flagged + quarantined"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-DOC-010]
---

## §1 — Statement (BCP-14 normative)

1. Documents imported from third-party providers (DocuSign, Adobe Sign, etc.) **MUST** have their signatures verified at import time against the issuing CA chains.
2. Invalid signatures **MUST NOT** block the import but **MUST** flag the document as `signature_status: invalid` in the repository with explanation in the audit row.
3. Valid signatures **MUST** have the verification result (signer cert, timestamp, OCSP status) persisted alongside.
4. Imported docs **MUST NOT** be co-mingled with platform-signed docs in legal-validity queries — the lineage is preserved.
5. The import gate **MUST** reject malformed PDFs that fail basic structural parsing.

## §2 — Why this constraint

Imported docs are common (legacy systems, partners on different platforms). Without verification at import, we can't tell trusted from untrusted documents in the repository. The "flag, don't block" rule lets users still import problematic docs for archive purposes while making the trust state visible. The lineage preservation prevents legally-imprecise queries from mixing the two categories.

## §3 — Measurement

- Counter `doc_import_signature_status_total{status=valid|invalid|unsigned}`.
- Counter `doc_import_malformed_total` — surfaces upload-pipeline issues.
- Gauge `doc_repo_invalid_signature_count`.

## §4 — Verification

- Integration test (T) — valid + invalid + malformed imports; assert correct handling.
- Pen test (T, quarterly) — adversarial imports.
- Lineage test (T) — legal-validity query excludes invalid-signature imports.

## §5 — Failure handling

- Invalid signature on import → flag + audit; user can still archive.
- Malformed PDF → reject + clear error.
- Lineage breach (invalid doc surfaces in legal query) → sev-2; query logic bug.

---

*End of NFR-DOC-009.*
