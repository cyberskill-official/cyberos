# `docs/srs/` — CyberOS System Requirements Specification

The SRS is the technical specification derived from the PRD — schemas, contracts, system behaviours.

| File | Purpose |
| --- | --- |
| [`SRS.md`](SRS.md) | The SRS itself (canonical Markdown source of truth as of 2026-05-12). |
| [`SRS.docx`](SRS.docx) | Last `.docx` snapshot. Retained for distribution but not the working copy — edit `SRS.md` and regenerate the docx via the back-conversion script when a Word-format export is needed. |
| [`CHANGELOG.md`](CHANGELOG.md) | Daily landing log for SRS changes. |

## Cross-references

- **PRD** (Product brief): [`../prd/`](../prd/)
- **AGENTS protocol** (memory layer): [`../memory/`](../memory/)
- **Contracts** (artefact schemas including `srs@1`): [`../contracts/srs/`](../contracts/srs/)

## How to update

1. Edit `SRS.md` directly.
2. Append a dated entry to `CHANGELOG.md`.
3. If the SRS change requires a corresponding update to the memory protocol, write the cross-layer note in `../memory/SRS.CHANGELOG.md` (if exists).
4. (Optional) regenerate `SRS.docx` from `SRS.md` if a Word export is needed for distribution.

## Folder history

- **2026-05-12** — converted `SRS.docx` to canonical Markdown `SRS.md` (task #65). Back-conversion script preserved for occasional docx export.
- **2026-05-12** — moved from `docs/CyberOS-SRS.docx` + `docs/CyberOS-SRS.CHANGELOG.md` (Batch D folder cleanup).
