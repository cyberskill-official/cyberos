# `docs/prd/` — CyberOS Product Requirements Document

The PRD is the authoritative product brief for CyberOS.

| File | Purpose |
| --- | --- |
| [`PRD.docx`](PRD.docx) | The PRD itself (Word doc; edit in place). |
| [`CHANGELOG.md`](CHANGELOG.md) | Daily landing log for PRD changes. Format inspired by Keep a Changelog, date-stamped. |

## Cross-references

- **SRS** (System Requirements Spec): [`../srs/`](../srs/)
- **AGENTS protocol** (memory layer): [`../memory/`](../memory/)
- **Skills layer**: [`../skills/`](../skills/)
- **Contracts** (artefact schemas including `prd@1`): [`../contracts/prd/`](../contracts/prd/)

## How to update

1. Edit `PRD.docx` directly.
2. Append a dated entry to `CHANGELOG.md` describing the change + rationale.
3. If the change introduces a NEW source-tier pattern or new policy that the memory layer must enforce, also append a note to `../memory/PRD.CHANGELOG.md` (if exists) describing the cross-layer impact.

## Folder history

- **2026-05-12** — moved from `docs/CyberOS-PRD.docx` + `docs/CyberOS-PRD.CHANGELOG.md` (Batch D folder cleanup).
