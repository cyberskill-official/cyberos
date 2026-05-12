# `docs/srs/` — CyberOS System Requirements Specification

The SRS is the technical specification derived from the PRD — schemas, contracts, system behaviours.

| File | Purpose |
| --- | --- |
| [`SRS.docx`](SRS.docx) | The SRS itself (Word doc; edit in place). |
| [`CHANGELOG.md`](CHANGELOG.md) | Daily landing log for SRS changes. |

## Cross-references

- **PRD** (Product brief): [`../prd/`](../prd/)
- **AGENTS protocol** (memory layer): [`../memory/`](../memory/)
- **Contracts** (artefact schemas including `srs@1`): [`../contracts/srs/`](../contracts/srs/)

## How to update

1. Edit `SRS.docx` directly.
2. Append a dated entry to `CHANGELOG.md`.
3. If the SRS change requires a corresponding update to the memory protocol, write the cross-layer note in `../memory/SRS.CHANGELOG.md` (if exists).

## Folder history

- **2026-05-12** — moved from `docs/CyberOS-SRS.docx` + `docs/CyberOS-SRS.CHANGELOG.md` (Batch D folder cleanup).
