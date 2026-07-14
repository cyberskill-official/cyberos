# Module docs content refresh - findings for operator resolution (2026-07-11)

A genuine line-by-line refresh of all 24 module pages (six parallel editors, each verifying claims against the real code under a no-invention rule). Stale facts were fixed directly: module count 22/23 -> 24 everywhere, Apache AGE removed, dead domains repointed to os.cyberskill.world/docs, /web/ paths dropped, VN overtime cap standardised to "200 hours/year, up to 300 with employee consent and MoLISA notification", crate names and source-file paths corrected to the real tree, and the payslip/cliff/horizon find-replace artifacts cleaned. Prior 2026-07-10 contradictions that had canonical answers were resolved in place.

What remains needs your ruling - each is a decision the editors were right not to invent.

## Real content decisions

- PLUGIN page identity (`modules/plugin/docs/index.md`). The page describes a specced-but-unbuilt cross-runtime distribution/marketplace module (pack-once-emit-many, OCI marketplace, OAuth-PKCE, commands `/cyberos-run` `/cyberos-memory` `/cyberos-skill-list` `/cyberos-route`). The plugin that actually ships (`tools/cyberos-init`) exposes `/init`, `/update`, `/changelog`, `/help` plus the `ship-tasks` skill. These are two different artifacts. Decide: retarget the page to the shipped cyberos-init plugin, or keep it as the distribution-module spec and add a line saying the shipped plugin lives at tools/cyberos-init.

- MCP gateway transport - gRPC (doc) vs HTTP (code). The design sections still describe per-module gRPC backends, a protobuf surface, and `grpc://` endpoints; the shipped `cyberos-mcp-gateway` registers and forwards over HTTP (`reqwest`, `POST /v1/mcp/register`, `http(s)://` endpoints only). The summary table and materials were aligned to HTTP; the substantive design sections were left. Decide: rewrite them to the HTTP/JSON-RPC model (recommended - gRPC looks like stale design, since per-module MCP servers speak HTTP by spec), or confirm gRPC is a real future.

- MCP gateway discovery endpoint. Doc references `/.well-known/mcp`; the router serves `/.well-known/oauth-authorization-server` + `/.well-known/oauth-protected-resource` (RFC 8414/9728, how MCP 2025-11-25 does discovery). Recommend replacing `/.well-known/mcp` with the PRM endpoints.

- CUO persona-slug convention (`modules/cuo/docs/module.md`). The §2/§4/§5 tables use short slugs (`ceo`, `cto`, `cro-revenue`, ...); the real on-disk folders and the appendices use full titles (`chief-executive-officer`, `chief-revenue-officer`, ...). The §7.5 example and the acronym count were fixed; the 48-row catalog was left (60+ references, a mechanical rewrite). Recommend a dedicated pass to rewrite the slug columns to the disk-true full-title form.

- Workflow count. proj was made undated ("47 personas, 220+ workflows"); the live repo has 224 workflow files; `modules/cuo/docs/module.md` still says 194. Recommend reconciling cuo to the live figure or the same undated phrasing.

- PORTAL phase label. The KB dependency diagram tags PORTAL as P2; the PORTAL page is authoritatively P4 long-term. Recommend relabelling the KB node to P4 unless "client KB views" is a deliberately earlier slice.

## Dated snapshots vs live totals (low stakes)

- skill appendices record "104 author+audit pairs / 208 bundles / 108 contracts" as of a dated session log; the live tree has 113 authors / 111 audits / 238 bundles / 108 contracts. Kept as dated snapshots (they are explicitly timestamped). Decide whether to add one current-state line with the live totals.

- KPI-section intros across the vision modules stated counts that trailed their tables; the editors set them to the real row counts (e.g. 9 -> 14, 10 -> 15, 11 -> 20). If any intended count differs, it is a one-line fix per page.

## Corpus-wide convention (not applied this pass)

- Periods and commas inside closing quotes are uniform across the whole migrated corpus. The refresh did not flip only these 24 pages, to avoid desyncing them from the other global/reference pages. If you want the logical-quote convention applied, it is a corpus-wide pass (all pages at once), not a per-module change.
