# `docs/prd/` — CyberOS Product Requirements Document

The PRD is the authoritative product brief for CyberOS.

| File | Purpose |
| --- | --- |
| [`PRD.md`](PRD.md) | The PRD itself (canonical Markdown source of truth as of 2026-05-12). |
| [`PRD.docx`](PRD.docx) | Last `.docx` snapshot. Retained for distribution but not the working copy — edit `PRD.md` and regenerate the docx via the back-conversion script when a Word-format export is needed. |
| [`CHANGELOG.md`](CHANGELOG.md) | Daily landing log for PRD changes. Format inspired by Keep a Changelog, date-stamped. |

## Cross-references

- **SRS** (System Requirements Spec): [`../srs/`](../srs/)
- **AGENTS protocol** (memory layer): [`../memory/`](../memory/)
- **Skills layer**: [`../skills/`](../skills/)
- **Contracts** (artefact schemas including `prd@1`): [`../contracts/prd/`](../contracts/prd/)

## How to update

1. Edit `PRD.md` directly.
2. Append a dated entry to `CHANGELOG.md` describing the change + rationale.
3. If the change introduces a NEW source-tier pattern or new policy that the memory layer must enforce, also append a note to `../memory/PRD.CHANGELOG.md` (if exists) describing the cross-layer impact.
4. (Optional) regenerate `PRD.docx` from `PRD.md` if a Word export is needed for distribution.

## Folder history

- **2026-05-12** — converted `PRD.docx` to canonical Markdown `PRD.md` (task #64). Back-conversion script preserved for occasional docx export.
- **2026-05-12** — moved from `docs/CyberOS-PRD.docx` + `docs/CyberOS-PRD.CHANGELOG.md` (Batch D folder cleanup).
