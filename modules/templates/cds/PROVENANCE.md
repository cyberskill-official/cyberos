# CDS vendoring provenance (TASK-TPL-001)

| field | value |
|---|---|
| source repo | https://github.com/cyberskill-official/design-system |
| design-system version | v1.3.0 (repo README snapshot) |
| @cyberskill/tokens package version | 0.1.0-prototype (design-system repo v1.3.0) |
| commit | 7231866de7df54c3950964711c67604235afd641 |
| copied | 2026-07-12 |
| files | tokens.css <- packages/tokens/dist/css/tokens.css; glass.css <- packages/react/src/glass.css |

Re-vendor procedure: clone the repo at the release tag, copy the two files verbatim, update this table (version + commit + date), run `tools/docs-site/tests/test_templates_module.sh` (byte-match asserts read this table's commit to fetch nothing - the test compares against these vendored bytes as the pinned truth; upstream drift is adopted ONLY by editing this file in the same commit).
