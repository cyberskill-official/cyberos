# Changelog — CyberOS-AGENTS.md

All notable changes to **CyberOS-AGENTS.md** are documented here, day by day.

This document does **not** carry an inline version marker — see CyberOS-AGENTS.md §0.2 (no-inline-version rule for design docs). Improvements land continuously; this changelog is the canonical record. Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), but date-stamped rather than version-stamped.

---

## 2026-05-06 (later evening) — Bundle K TIER 1: Deprecate `.protocol-signing-key` file

### Changed
- **§0.5 TOFU paragraph** — removed the `cyberos/.protocol-signing-key` reference. New wording: *"Trust establishment is TOFU: the first fingerprint enters the manifest via explicit user paste from any trusted out-of-band source — a CyberSkill-signed announcement, a verified org-wide secrets manager, an in-person fingerprint exchange, or any equivalent. **Pre-BRAIN-module-P1, no canonical out-of-band source is mandated by this protocol** (the canonical mechanism lands when P1 ships)."*

### Removed
- **`cyberos/.protocol-signing-key`** (deprecated) — overwritten with a tombstone-style deprecation marker referencing DEC-094 v2 / DEC-105 / REF-026. The cowork sandbox can't `rm` files outside `.cyberos-memory/`; user can manually delete from local clone if desired.

### Updated
- **DEC-094 v=1 → v=2** — appended History entry documenting the Bundle K deprecation. The original "signing_keys bullet" prose remains in v1 history; the v2 prose acknowledges the file approach was deferred.
- **README.md Part 6 (Protocol distribution)** — removed the "baked into the cyberos repo" sentence; replaced with the post-K wording matching §0.5.

### Real-world trigger
Stephen flagged the file as friction: *"is there any way that no need one more separate file .protocol-signing-key?"* Honest analysis: it was placeholder weight. No real CyberSkill signing key exists yet (BRAIN module P1 hasn't shipped); the file documented an aspiration rather than enforcing real trust. Stephen picked Option A (delete now, defer real distribution mechanism to P1) over Options B (embed in AGENTS.md frontmatter) and C (keep file; defer decision).

### Why TIER 1 only
Single paragraph rewrite + one file deprecation + one DEC version bump + one README sentence. No new mechanism; no schema change; no audit-row format change. Pure surface-area reduction.

### Schema impact
None. `manifest.protocol.signing_keys[]` array remains in §6 unchanged — it just no longer has a canonical pre-P1 population source. Auto-§8.7 post-upgrade scan per Bundle J is expected to report 0c/0w because nothing changed at the §5.1 frontmatter level.

### AGENTS.md canonical SHA
Pre-K `sha256:1a55e8b…2edb` → post-K (computed at write).

### BRAIN entries
DEC-094 v=2 (signing-key-file approach deferred to P1), DEC-105 (Bundle K decision), REF-026 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 6 / Part 8 (Bundle K added as the twelfth real-world trigger; first to REMOVE surface area).

---

## 2026-05-06 (later evening) — Bundle J TIER 1: Auto-trigger §8.7 after protocol_upgrade + uppercase BRAIN in trigger phrases

### Added
- **§0.5 step 4** — every successful `op:"protocol_upgrade"` now auto-triggers a §8.7 self-audit pass immediately after the manifest pin and the protocol_upgrade audit row. This is the post-upgrade migration check: schema validate (phase 1) catches memories failing the new §5.1; supersedes-graph integrity (phase 2) catches dangling relationships if scopes were renamed; resource caps (phase 6) catches new field additions pushing files over §5.5 limits. Findings surface per §8.7 severity routing. Skip only with explicit phrase *"skip post-upgrade scan"* (logged as `op:"skipped-by-user"`).
- **§6 manifest** — `health_check_policy.post_upgrade_phrase` field. Default value: *"rescan BRAIN"* (uppercase BRAIN per §0.3 / Bundle H). Manually triggers the same scan as the auto-flow.
- **§8.7 "Post-upgrade scan" subsection** — distinguishes the post-upgrade flavour from routine on-demand health-checks. Identical mechanics; report file named `meta/health/<YYYY-MM-DD>-<sha>-postupgrade.md` to mark provenance. The §14 block reports it as a post-upgrade scan.

### Changed
- **`manifest.health_check_policy.on_demand_phrase` default** — *"run brain healthcheck"* → *"run BRAIN healthcheck"* (uppercase BRAIN per §0.3 / Bundle H consistency).
- **`manifest.health_check_policy.diagnostic_verbs[]` defaults** — entries mentioning BRAIN switched to uppercase: *"check brain"* → *"check BRAIN"*; *"show brain"* → *"show BRAIN"*; *"view brain"* → *"view BRAIN"*. Lowercase versions explicitly NOT diagnostic triggers (they're anatomy/metaphor per §0.3).
- **§1 step 2** — diagnostic-verb list updated to match the new manifest defaults; added a one-sentence note: *"verbs that mention 'BRAIN' use uppercase per §0.3 (case-sensitive alias); lowercase 'brain' verbs are NOT diagnostic triggers."*

### Real-world trigger
Stephen asked: *"can we auto trigger scan and re-arrange/refine the .cyberos-memory after AGENTS.md update, because there maybe breaking changes or rules that need to adapt, and how to manual trigger that?"* Plus reinforcement: *"for manual i want 'run BRAIN healthcheck' instead"* (uppercase BRAIN). Bundle J answers both: §8.7 already had the schema-validate check that catches new-schema-failures; auto-triggering §8.7 after every protocol_upgrade was a one-step amendment to §0.5. The uppercase-phrase fix completes Bundle H's case-sensitivity work — three places still had lowercase "brain" in default trigger phrases that should have been uppercase for consistency.

### Why TIER 1 only
Single sentence-and-a-half §0.5 amendment + 4 default-value updates + one new §8.7 paragraph. No new ops, no new scopes, no new mechanism. The §8.7 phase-1 schema-validate already does the migration check — Bundle J just wires it into the post-upgrade flow automatically.

### What this does NOT change
- The §8.7 checks themselves (still six checks; same severity buckets; same `meta/health/` location).
- The audit ledger format and chain semantics — unchanged.
- Existing `on_demand_phrase` users with lowercase phrases configured — those are project-level overrides; only the default ships uppercase. Existing manifests are not migrated automatically.

### Migration note for cyberos's own manifest
Cyberos's running `manifest.health_check_policy.on_demand_phrase` updated to "run BRAIN healthcheck" as part of this Bundle's manifest re-pin. `diagnostic_verbs[]` entries also uppercased.

### AGENTS.md canonical SHA
Pre-J `sha256:7e229a2…2545d` → post-J (computed at write).

### BRAIN entries
DEC-104 (auto-trigger §8.7 + uppercase BRAIN phrases decision), REF-025 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (Bundle J added as the eleventh real-world trigger).

---

## 2026-05-06 (later evening) — Bundle I TIER 1: Compact §14 format gated by operational_mode

### Added
- **§14.1 Compact format** (default for `operational_mode: normal`) — `Δ Changes:` block showing only paths with actual changes; `Status:` block with conflicts/drift/shallow/sync/health one-liner; `unchanged:` roll-up line. Analysis-only turns collapse `Δ Changes:` to a single line `(no mutations this turn — <justification>)`.
- **§14.2 Full format** (default for `operational_mode: verbose | debug | maintenance`) — pre-Bundle-I per-scope-explicit format retained. `maintenance` mode prepends a `🔧 MAINTENANCE` banner with `maintenance_session_id`.
- **§14.4 Authority clarifier** — the audit ledger is the authoritative record; the §14 block is human-readable summary; format changes per `operational_mode` do not affect audit chain integrity.

### Changed
- **§14 opening paragraph** — now declares the two-format split and points at `manifest.operational_mode` as the discriminator.
- **§14.3 Coverage stat for ingestion ops** — unchanged content; renumbered from prose-paragraph to its own subsection for symmetry.

### Real-world trigger
Stephen flagged real readability friction post-Bundle-H: *"sometime this section so long and hard to read, is there any way to present it more verbose & human easier read?"* Surveyed prior turn outputs — every §14 block had ~14 lines, ~9 of which read "no change" verbatim. Signal lost in noise. The `operational_mode` field (added Bundle C) was the right discriminator — it already exists; reuse for rendering avoided new mechanism. Third refinement from real-world use; first that targets human-UX rather than protocol semantics.

### Why TIER 1 only
Single section rewrite; reuses existing `operational_mode` mechanism; no new fields, no new ops, no new scopes. Clean rollback path via the verbatim archive.

### What this does NOT change
- Audit ledger format and chain semantics — unchanged.
- §14 mandatory status (still required after every substantive reply).
- Coverage stat for ingestion ops (still mandatory; just renumbered §14.3).
- Per-mode behaviour outside §14 (DEBUG mode banners per §8.7 still apply; MAINTENANCE mode permissions per §8.8 unchanged).

### AGENTS.md canonical SHA
Pre-I `sha256:fe0773c…251aa` → post-I (computed at write).

### BRAIN entries
DEC-103 (compact-§14-by-operational_mode decision), REF-024 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (Bundle I added as the tenth real-world trigger; first targeting human-UX).

---

## 2026-05-06 (later evening) — Bundle H TIER 1: Strict uppercase BRAIN alias (§0.3)

### Changed
- **§0.3 first paragraph** — added explicit case-sensitivity clause: *"(literal uppercase B-R-A-I-N; case-sensitive — lowercase 'brain' does NOT trigger this alias)"*. The pre-H wording said *"the BRAIN"* / *"your BRAIN"* with implied capitals but didn't enforce it; a literal reader could have matched lowercase "brain" too.
- **§0.3** added a "Lowercase 'brain' is normal language" clarifier paragraph listing common lowercase usages (anatomy, metaphor, general topic) that explicitly do NOT trigger the alias. Includes an ambiguity-disambiguation rule: when context strongly implies memory-store but casing is lowercase, the agent asks a clarifying question rather than silently assuming.

### Real-world trigger
Stephen noticed: *"i notice that 'brain' still work? i want only 'BRAIN' will be understand as the memory, because some topic relate to human brain may trigger too, right?"* — confirmed that pre-H §0.3 didn't enforce case, leaving a small but real false-positive surface (lowercase "brain" in non-memory contexts could be misinterpreted). Second refinement from real-world use; Bundle G was the first.

### Why TIER 1 only
Single-paragraph change; narrowly scoped; closes the observed gap. No TIER 2/3 candidates surfaced.

### What this does NOT change
- §1 step 2's diagnostic-verb list (Bundle G) keeps lowercase phrases like "check brain", "show brain", "view brain". Those verbs trigger `PRISTINE-DIAGNOSTIC-HOLD` based on intent, NOT BRAIN-alias activation. The two mechanisms are independent.
- The case-sensitivity rule applies only to §0.3 alias activation; written prose elsewhere in the protocol can use either case for readability.

### AGENTS.md canonical SHA
Pre-H `sha256:3804334…f0ecb` → post-H (computed at write).

### BRAIN entries
DEC-102 (strict-uppercase BRAIN alias decision), REF-023 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (Bundle H added as the ninth real-world trigger; second from real-world use).

---

## 2026-05-06 (later evening) — Bundle G TIER 1: Diagnostic-verb carve-out for PRISTINE auto-bootstrap

### Added
- **§1 step 2 carve-out** — auto-bootstrap is silent UNLESS the user's current-turn message contains a recognised diagnostic verb (default list: `healthcheck`, `status`, `inspect`, `audit`, `check brain`, `show brain`, `view brain`, plus configured `on_demand_phrase`). When intent is diagnostic AND state is `PRISTINE`, the agent enters `PRISTINE-DIAGNOSTIC-HOLD` and surfaces the absent state instead of bootstrapping.
- **§13.0 `PRISTINE-DIAGNOSTIC-HOLD` row** — sub-state of `PRISTINE`. Agent surfaces what would be created by §13.1 and waits for explicit consent (`bootstrap and continue`, `just bootstrap`, or any task-oriented instruction). Does NOT write during this state.
- **§6 manifest extension**: `health_check_policy.diagnostic_verbs[]` — array of strings; project-level override of the default verb list.

### Real-world trigger
A fresh Cowork session at `sale-noti/` (the first downstream consumer of the protocol post-Bundle-F) ran `healthcheck` against a `PRISTINE` BRAIN. The agent correctly held off on silent auto-bootstrap, reasoning that bootstrapping mid-diagnostic would change the very state being inspected. It surfaced this as an §0.4 candidate for upstream propagation. Stephen approved upstreaming the refinement so future downstream projects don't re-encounter the friction. **This is the first refinement triggered by a real downstream project's actual use of the protocol** — the §0.4 propose-then-adopt loop firing in the wild rather than during meta-protocol design.

### Changed
- AGENTS.md canonical SHA: pre-G `sha256:f7f3934…f4f1b7` → post-G (computed at write time).

### BRAIN entries
DEC-101 (diagnostic-verb carve-out decision), REF-022 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (now lists Bundle G as the eighth real-world trigger; first one originating from a downstream project).

---

## 2026-05-06 (evening) — Bundle F: Comprehensive audit-fix pass + §0.6 related-files rule

### Added
- **§0.6 Related-files update rule** (sev-1) — every successful `op:"protocol_upgrade"` MUST be followed in the same chat turn by updates to: CHANGELOG (dated entry), README (any tracked Part), cross-linked FACT memories (e.g., FACT-004), and implementation files (e.g., `brain_writer.py` for §7.2; `.protocol-signing-key` for §0.5). Order of operations enumerated. Self-detection extension at §8.7 phase 1 reserved for Bundle G.
- **§7.5 `op:"corrects"` vs `correction_to` field** — distinguishes the two mechanisms. `op:"corrects"` is its own audit row for content correction (the world changed); `correction_to` is a field on any op marking that THIS row corrects the agent's own prior action. Rule: every `op:"corrects"` MUST have `correction_to` set; non-corrects ops MAY set it for self-correction.
- **§8.1 / §8.2 / §8.3 / §8.4 / §8.5 explicit subsection headers** — phases 1-5 of consolidation now have their own subsection numbers, matching §8.6 / §8.7 / §8.8 already-explicit subsections. Closes the §11.5-references-§8.5 dead reference.

### Fixed (TIER 1 — bugs / stale claims)
- **§0 line 22**: "§0 through §16" → "every section of `AGENTS.md` from §0 to the end" (was stale since Bundle A added §17).
- **§5.1 heading**: "27 fields" → "28 fields" (was stale since Bundle A added `sync_class`).
- **§8 heading**: "5 phases" → "7 phases" with explicit §8.1–§8.5 subsection headers (was stale since §8.6 + §8.7 added).
- **§8.7 step 4**: chain hash formula updated to match Bundle D §7.2 — now uses `row_without_chain_or_prev_chain`; clarifies LINK integrity is authoritative and hash recomputation is INFO-severity. (Was a bug — old §8.7 wording would have caused implementations to compute wrong hashes.)
- **§4.7 orphan-manifest pairing**: now accepts `consolidation_run | protocol_upgrade | protocol_rollback | session.end` as valid terminators (was a real bug — old wording would have flagged every Bundle's protocol_upgrade as crash-mid-consolidation and frozen writes).
- **§9.7 Delete row**: removed undefined "30-day legal hold" language; replaced with §4.6 cross-reference.
- **§9.7 Privacy row**: cites §17 sync_class (the actual mechanism) and §6 exclusion_rules (for ingestion-blocking).
- **§11.5 step 5**: "(§8.5)" — now resolves to the explicit §8.5 subsection added above.
- **§11.6 declares M&A-only schema extensions**: `original_chain` field on rebased audit rows + `manifest.imported_sources[]` array — both formally defined, with `INCOMPATIBLE:<field>` exemption when `imported_sources[]` is non-empty.
- **§17.5 `manifest.actor_keys`**: clarified as aspirational — to be added to §6 schema via §0.5 protocol upgrade at BRAIN module P1, not yet present.

### Fixed (TIER 2 — stale or inconsistent)
- **§3 layout**: now lists `meta/protocol-history/` (per §0.5) and `meta/health/` (per §8.7) as first-class subdirectories.
- **§13.1 step 2**: `tenant.id`/`owner.id` `null` (not `""`) when unknown.
- **§16 Tie-breakers**: "flag for next consolidation" → `op:"warn"` (matches post-Bundle-C vocabulary).
- **§0.2 bullet**: "schema_version" → "manifest field outside §6 schema" (the `schema_version` field was removed 2026-05-04 afternoon; the bullet was stale).

### Fixed (TIER 3 — compression / consolidation)
- **§0.5 "Forbidden by §0.2" paragraph** → one cross-reference sentence.
- **§4.10 forbidden-tool patterns** → compressed from five bullets to one parenthetical (the principle is "walk sequentially; no sampling"; the specific tools were examples).
- **§4.1 step 5** → absorbs §11.7's path constraints (length cap, case-collision, Windows-illegal chars). §11.7 reduced to a one-line cross-reference.
- **§9.4 project-specific examples** → generalised to "specific opt-in topics live in `meta/opt-ins.md` per project" (matches `feedback-no-project-specific-examples-in-universal-docs.md` standing rule).

### AGENTS.md canonical SHA
Pre-F `sha256:f9328b7…cb1022` → post-F `sha256:f7f3934…f4f1b7`.

### Real-world trigger
Stephen requested: *"check whole CyberOS-AGENTS.md content to find things that can be refine/compress/combine/merge/drop..."* Comprehensive audit surfaced 19 issues across three tiers. User adopted all three tiers in one bundle. The §0.6 related-files update rule was added at user's reinforcement: *"remember always update readme and changelog after AGENTS.md changes."*

### Pre-F archive
`meta/protocol-history/AGENTS-sha256-f9328b7…cb1022.md` (verbatim, captured at session.start before any edits).

### BRAIN entries
DEC-100 (audit-fix pass + related-files rule), REF-021 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (now lists Bundle F as the audit-cleanup pass).

---

## 2026-05-06 (evening) — Bundle E TIER 1: Three-way protocol-conflict handling (§0.5 + §13.0)

### Added
- **§0.5 "Three-way conflict (loaded ≠ pinned ≠ upstream)" subsection** — defines the case where loaded SHA `Y`, pinned SHA `X`, and upstream-available SHA `Z` all differ. Agent enters `INCOMPATIBLE:three-way-protocol-conflict` state, refuses to apply upstream, surfaces a structured prompt with three explicit user options (revert local; approve local as upgrade; manual three-way merge then approve via the standard §0.5 phrase). No automated merge.
- **§13.0 state classifier row**: `INCOMPATIBLE:three-way-protocol-conflict`. Same freeze-write handling as 2-way `protocol-sha256-mismatch`.

### Changed
- AGENTS.md canonical SHA: pre-E `sha256:b4042a6…cacce3` → post-E `sha256:f9328b7…cb1022`.

### Real-world trigger
Stephen asked (post-cascade): *"did we take care of the case when local BRAIN conflict with upstream BRAIN when update?"* Honest diagnosis: the post-cascade §0.5 mechanism handled the 2-way mismatch (loaded vs pinned, scenario A) and the clean upstream upgrade (scenario B), but did NOT handle the 3-way case (scenario C) — a user with hand-edited AGENTS.md running "check for protocol updates" would have had local edits silently overwritten. TIER 2 (multi-actor protocol-version skew) and TIER 3 (key rotation operational flow) deferred — both gain operational relevance only when the BRAIN module's network surface ships at P1.

### Why TIER 1 only
- Closes the most immediate observed gap (silent overwrite of local hand-edits during upstream pull).
- Extends existing conservative §13.0 discipline (writes-frozen-until-explicit-resolution) from 2-way to 3-way without inventing new mechanisms.
- The three explicit options map cleanly onto existing §0.5 vocabulary.
- TIER 2 + TIER 3 are not currently load-bearing (no BRAIN module endpoint, no real signing key) — adopting them speculatively today would be bulk without proportional value.

### Operational note
Pre-E archive: `meta/protocol-history/AGENTS-sha256-b4042a6…cacce3.md` is **verbatim** (created during the 2026-05-06 rollback validation test per DEC-098). Bundle E inherits it as its pre-state archive without needing to re-create — full rollback support from Bundle D forward.

### BRAIN entries
DEC-099 (three-way protocol-conflict decision), REF-020 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 6 (Protocol distribution) — content unchanged today; will reference §0.5's three-way subsection when next revised.

---

## 2026-05-06 (evening) — Bundle D: Canonical-JSON tightening (§7.2 → RFC 8785 JCS)

### Changed
- **§7.2 Canonical JSON for hashing** — rewritten to cite **RFC 8785 (JSON Canonicalization Scheme, JCS)** as the authoritative algorithm. Previously underspecified ("keys sorted, compact separators, shortest IEEE-754") which permitted multiple legal interpretations. Now documents exact serialisation primitives:
  - Object key ordering: lexicographic on UTF-16 code units (RFC 8785 §3.2.3).
  - Whitespace: none anywhere; no trailing newline.
  - Separators: literal `,` and `:` bytes; no surrounding whitespace.
  - Strings: UTF-8, NFC-normalised, non-ASCII preserved verbatim (no `\uXXXX` escapes for non-control chars).
  - Numbers: ECMAScript `Number.prototype.toString` (shortest round-trip via IEEE-754 double); integers without trailing `.0`; **Python `1.0` MUST serialise as `1`, not `1.0`** (the most common cross-writer-version divergence).
  - Booleans/null: lowercase `true`/`false`/`null` only.
  - No duplicate keys.
- **Reference implementations named**: `rfc8785` PyPI package; `canonicalize` npm package. Hand-rolled `json.dumps(sort_keys=True, …)` MUST validate against JCS test vectors before being trusted to chain audit rows.
- **Cross-writer-version compatibility clarified**: the chain LINK invariant (`row[N].prev_chain == row[N-1].chain`) is the **authoritative** integrity guarantee. Hash *recomputation* across writer versions MAY fail (different writers emit different bytes for logically-identical rows); this is informational and surfaced at INFO severity in §8.7 self-audit, NOT a chain break.
- **Body exclusion clarified**: `canonical_json` receives `row_without_chain_or_prev_chain`; `prev_chain` is concatenated as raw bytes AFTER the canonical body.

### Real-world trigger
The 2026-05-06 cascade verifier (`outputs/verify_v2.py`) surfaced 149 pre-existing audit rows failing bit-perfect hash recompute against the new `brain_writer.py`, despite both writers nominally following pre-D §7.2. LINK integrity intact; recompute divergent. Surfaced as a TIER 1 §0.4 candidate at the end of the prior turn ("§7.2 is underspecified"); user adopted as Bundle D in the next turn.

### What this does NOT do
Pre-D rows remain hash-non-reproducible. The cardinal rule (additive-only) is preserved because pre-D rows are not retroactively touched. LINK integrity holds. Forcing a re-chain would invalidate any external exports already pinned to those chain values.

### AGENTS.md canonical SHA
Pre-D `sha256:7cd4a56…ad650a` → post-D `sha256:b4042a6…cacce3`.

### BRAIN entries
DEC-097 (canonical-json-rfc-8785 decision), REF-018 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (How to evolve the protocol safely — sixth real-world trigger).

---

## 2026-05-06 (evening) — Bundle C: Self-audit pass + DEBUG/MAINTENANCE modes (§8.7, §8.8)

### Added
- **§8.7 Self-audit pass** (sev-1) — sixth phase of consolidation; runs under `.lock`. Six checks: schema validate, supersedes-graph integrity, relationships-graph integrity, audit chain integrity (end-to-end recompute), orphan files, resource caps. Three severity buckets: `CRITICAL` (freezes writes), `WARN` (surfaced), `INFO` (logged).
- **Three operational modes** via `manifest.operational_mode`: `normal` (WARN/CRITICAL in §14 block); `debug` (every reject/revert/warn this session floats to top of next response as a banner); `verbose` (adds successful-op tracing).
- **§8.8 MAINTENANCE mode** (sev-0) — distinct from DEBUG; the safe version of "ROOT". Time-limited (1 hour or session end). Permits specific repair ops normally forbidden: chain rebuild, orphan tombstone, force-resolve conflict, manual rollback, frontmatter migration edit. Each repair requires per-op chat confirmation. Logged with `actor_kind: maintainer` + `maintenance_session_id`. NEVER bypasses §9.3 denylist or §4.2 content gate.
- **§6 manifest** — `operational_mode: "normal"` (default) and `health_check_policy: {on_session_end, on_demand_phrase}`.
- **§7.1 audit op enum** — `health_check`, `warn`, `drift_candidate`, `shallow_candidate`, `maintenance.start`, `maintenance.end`.
- **§14 end-of-response block** — new line: `health: <N critical | M warn | K info>; operational_mode: <…>`.
- **`meta/health/`** — new directory; stores deterministic health-check reports keyed by `<YYYY-MM-DD>-<sha>`.

### Deferred
- **TIER 2 — Org-level escalation channel** — when the BRAIN module ships at P1, CRITICAL + aggregated WARN forward to a CyberSkill admin channel. Privacy boundary: only metadata escalates; never memory content.

### Changed
- AGENTS.md canonical SHA: pre-C `sha256:8025a96…b13d65` → post-C `sha256:7cd4a56…ad650a`.

### Real-world trigger
Stephen asked (2026-05-06): *"Can the BRAIN audit itself? While users are using the BRAIN and unexpected issues happen, I should be notified so I can fix it asap. For now maybe we can use DEBUG or ROOT mode."* Diagnosis: pre-C protocol had partial self-audit elements (§4.7, §8.6, §13.0, §0.4, §1.10) but no integrated full-store integrity pass, no notification channel beyond the easily-missed §14 block, and no clear separation between read-side verbosity (DEBUG) and write-side repair authority (MAINTENANCE). Conflating the two risks the Linux-root footgun pattern.

### BRAIN entries
DEC-096 (self-audit + DEBUG/MAINTENANCE decision), REF-017 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 7 (Self-audit & operational modes).

---

## 2026-05-06 (evening) — Bundle A: Sync-class boundary (§17)

### Added
- **§17 Personal vs shared memory boundary** — declares the four sync classes (`local-only`, `publishable`, `shared`, `client-visible`), per-scope defaults table (§17.2), per-subject identity model (§17.3 — subject not machine is the trust anchor), absorb-then-discard offboarding semantics (§17.4), publish-flow forward reference (§17.5 — mechanism deferred to BRAIN module P1), and explicit out-of-scope list (§17.6 — wire protocol, ACL, conflict mechanism, key rotation all live in the BRAIN/PORTAL modules, not here).
- **§5.1 frontmatter** — 28th permitted field: `sync_class: local-only | publishable | shared | client-visible`. Per-file overrides allowed.
- **§14 end-of-response block** — new line: `sync class summary: <N local-only | M publishable | K shared | J client-visible>`.

### Changed
- **§11.8** — last sentence rewritten to clarify scope: "This protocol governs the personal layer of the BRAIN. Continuous multi-machine sync of shared scopes happens through the runtime BRAIN module (FACT-004 Layer 2), not via filesystem replication." Closes the §11.8↔FACT-004 contradiction (was: "Concurrent multi-machine editing of the same project is unsupported; pick one authoritative machine" — read literally, that contradicted FACT-004's "CRDT sync across machines" claim).
- AGENTS.md canonical SHA: pre-A `sha256:6e993e3…b4797b` → post-A `sha256:8025a96…b13d65`.

### Real-world trigger
Stephen asked (2026-05-06): *"It's working as personal memory for one person. But each person will contribute to CyberSkill activities (via CyberOS), so it needs to serve both personal-based memory as well as CyberOS's memory. Should we think about that now?"* Surfaced two pre-existing gaps: §11.8↔FACT-004 contradiction (would fire as soon as a second laptop joins); personal-vs-org boundary was implicit so every memory written today was being classified by accident. Resolution: lock the boundary now via the four sync classes; defer mechanism (signing, wire protocol, ACL) to the runtime BRAIN module.

### User answers driving the design
Q1 *CyberSkill one tenant?* → publisher today, multi-tenant SaaS at P3+ supported by per-tenant region pinning. Q2 *project/ flows to org?* → yes, defaults to `shared` (CyberOS architecture is the company's product). Q3 *clients consume a slice?* → yes, fourth class `client-visible`. Q4 *offboarding?* → absorb knowledge, discard fragments. Q5 *per-machine or per-person?* → per-person identity (subject is trust anchor; multiple machines mirror through org BRAIN).

### BRAIN entries
DEC-095 (sync-class boundary decision), REF-016 (refinement record), FACT-004 v2 (Layer 1 paragraph rewritten to cite §17 instead of bare "CRDT sync"; closes the contradiction).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 5 (Personal vs org: the four sync classes).

---

## 2026-05-06 (evening) — Bundle B: Protocol distribution policy (§0.5)

### Added
- **§0.5 Protocol update policy** (sev-0) — defines canonical SHA computation, manifest pin via `manifest.protocol.sha256`, session-start tripwire, the explicit chat-turn approval phrase *"approve protocol upgrade to `<sha256:…>`"*, archive-then-update flow, rollback path, signed upstream release flow with TOFU trust establishment, bootstrap behaviour, §0.2 forbidden list.
- **§6 manifest** — `protocol` block: `{sha256, approved_at, approved_by, loaded_path, signing_keys[], last_checked_at}`.
- **§7.1 audit op enum** — `protocol_upgrade`, `protocol_rollback`.
- **§13.0 state classifier** — `INCOMPATIBLE:protocol-sha256-mismatch` (canonical SHA mismatch with manifest pin → freeze writes; require chat-turn approval phrase to resolve).
- **§13.1 bootstrap** — step 12 (auto-pin canonical SHA at first run, no prompt) and step 13 (seed `meta/protocol-history/` for rollback archive).
- **`meta/protocol-history/`** — new directory; stores verbatim AGENTS.md archives keyed by SHA suffix; exempt from §5.1 frontmatter (these are protocol-doc archives, not memories; integrity is content-addressable via SHA).

### Changed
- AGENTS.md is now content-addressable. Pre-B canonical SHA `sha256:560a489…1600fc`. Post-B canonical SHA `sha256:6e993e3…b4797b`.

### Real-world trigger
Stephen asked (2026-05-06): *"AGENTS.md behaves like global instructions when copied to local machine. Is there any way to force-sync it with CyberOS's AGENTS.md to make sure all distributed BRAINs are updated when CyberOS has a new BRAIN version?"* Surfaced two pre-existing gaps: AGENTS.md was silent on its own update flow (no tripwire for hand-edits, host-platform silent updates, or accidental drift); "force sync" would defeat §0.2 (the same gate that protects from prompt injection would also block forced sync). Resolution: layered authenticity (Ed25519 signatures, deferred to TIER 2 / BRAIN module P1), authorization (chat-turn approval phrase per §0.2), and auditability (`op:"protocol_upgrade"` rows + `meta/protocol-history/` archive).

### BRAIN entries
DEC-094 (protocol-update-policy decision), REF-015 (refinement record). Both adopted in chat per §0.4.

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 6 (Protocol distribution).

---

## 2026-05-06 (evening) — README on-ramp shipped (informational; no AGENTS.md edits)

### Added
- **`docs/CyberOS-AGENTS.README.md`** — comprehensive 12-part reader's guide & evolution manual. Sections cover the mental model (Parts 1–4), the personal-vs-org sync-class boundary (Part 5), protocol distribution (Part 6), self-audit & operational modes (Part 7), the safe-evolution playbook with additive-only rules and the §0.4 propose-adopt-record loop (Part 8), common mistakes (Part 9), troubleshooting decision tree (Part 10), reading-order guide for AGENTS.md (Part 11), and glossary (Part 12).

### Why it's a CHANGELOG entry but no AGENTS.md edits
- The README is a **companion** doc, not part of the protocol itself. Editing it never triggers the §0.5 protocol-upgrade approval flow.
- The README captures decisions adopted in the same session (sync_class TIER 1, protocol-distribution TIER 1+3, self-audit TIER 1+3) that are *pending implementation* in AGENTS.md. The README explains the target state; the AGENTS.md cascade lands separately.
- This follows the same "informational; no AGENTS.md edits" pattern as the 2026-05-06 skill-registry entry below.

### Pending cascade (next coordinated batch)
- AGENTS.md edits: §0.5 protocol update policy, §6 manifest extension (`protocol`, `signing_keys`, `operational_mode`), §7.1 op enum (`protocol_upgrade`, `protocol_rollback`, `health_check`, `warn`), §8.7 self-audit pass, §13.0 state classifier (`INCOMPATIBLE:protocol-sha256-mismatch`), §13.1 bootstrap auto-pin, §14 block additions (`sync class summary`, `health check`), §17 personal-vs-shared memory boundary with 4-class sync_class.
- Memory writes: DEC-094 (sync_class boundary), REF-015 (sync_class refinement), DEC-095 (protocol update policy), REF-016 (protocol distribution refinement), DEC-096 (self-audit + DEBUG/MAINTENANCE modes), REF-017 (self-audit refinement), FACT-004 cross-link update (closes the §11.8↔CRDT contradiction).
- Once landed, this CHANGELOG gets a separate dated entry per refinement bundle.

### Cross-link
- See `docs/CyberOS-AGENTS.README.md` Part 8 for the reasoning behind the additive-only evolution rule and the propose-adopt-record loop.

---

## 2026-05-06 — Skill-registry v0.2.0 (informational; no AGENTS.md edits)

### Context

The skill registry at `cyberos/docs/skills/` shipped v0.2.0 with:
- Skills↔contracts namespace split (DEC-090).
- Dual-mode invocation + exposability frontmatter (DEC-091).
- Self-audit + auto-refinement at skill level (DEC-092).
- Manual fine-tune playbook (DEC-093).
- Plus the consolidated `README.md` wiki + the onboarding infographic.

### Why this is an AGENTS.md changelog entry but no AGENTS.md edits

- AGENTS.md governs the **BRAIN** (`.cyberos-memory/`) protocol — memory writes, the audit ledger at `audit/<YYYY-MM>.jsonl`, the consolidation cycle, the conflict-resolution graph.
- The skill registry's `genie.action_log` is a **separate** audit stream (the runtime's, per SRS §6.7) that records skill outputs. It chains independently from the BRAIN's ledger.
- The new skill-level `op:"self_refinement_proposal"` rows live in `genie.action_log`, not in the BRAIN. AGENTS.md §7.1's `op` enum is unaffected.
- The skill-level `self_audit` + `INVARIANTS.md` machinery is a **parallel** of AGENTS.md §0.4's standing rule, applied at the skill level rather than the protocol level. Same pattern, different surface.

### Cross-link

- See `cyberos/docs/skills/CHANGELOG.md` v0.2.0 for the registry-side detail.
- BRAIN entries DEC-090 / DEC-091 / DEC-092 / DEC-093 record the underlying decisions; REF-012 / REF-013 / REF-014 record the §0.4 refinement candidates surfaced during the design conversation.

---

## 2026-05-04 (evening, follow-up) — Validator discipline: fenced-code-block exemption + datetime-instance acceptance

### Changed
- **§4.3 file-content hygiene** — multi-frontmatter check now exempts content inside fenced code blocks (` ``` ` or `~~~`). Strip fenced spans before the secondary-block scan. Code-fenced examples of YAML frontmatter are legitimate Markdown content (common in skill / format / spec docs that show example `SKILL.md` or memory-file frontmatter) and must not trigger `multiple-frontmatter-blocks` rejection. Opening-block check unchanged. (DEC-087)
- **§5.2 timestamp validator row** — accept either an ISO-8601 string matching the existing regex OR a tz-aware language-native datetime instance. PyYAML and similar loaders auto-coerce ISO-8601 to native datetimes; `str(dt)` then renders with a space separator (`2026-05-04 21:13:29+07:00`) and fails the regex. Validators MUST handle both forms. Naive (tz-less) datetimes rejected as `naive-ts:<field>`. Offset and minute-granularity rules unchanged. (DEC-088)

### Real-world trigger
Surfaced during the skills-knowledge digest session (workbench/.cyberos-memory bootstrap, 2026-05-04 evening). Both failures hit on the very first memory-file write of a corpus of 12:
1. `spec.md` body legitimately contained `---`-delimited example SKILL.md frontmatter inside ```` ``` ```` fences. The §4.3 secondary-block scan triggered `multiple-frontmatter-blocks` rejection. Any session ingesting skill-format documentation, agent-protocol docs, or any spec that shows example frontmatter in code fences would have hit the same crash deterministically.
2. PyYAML's `safe_load` auto-parses ISO-8601 timestamps into `datetime.datetime` objects. The §5.2 validator's regex then ran on `str(dt)` which produces `2026-05-04 21:13:29+07:00` (space separator) instead of `2026-05-04T21:13:29+07:00` (T separator) and rejected its own valid output as `bad-ts:created_at`. Affects every Python implementation using PyYAML — i.e., effectively all of them.

Both refinements were proposed as Tier-1 (directly prevents observed failure) per §0.4 in the same response that surfaced them, and Stephen adopted both. The implementing patches in the session's local `.brain_writer.py` (a §4.4 atomic-write helper) are the reference implementations; both validators worked correctly against the remaining 11 memory files after patching.

## 2026-05-04 — Ingestion-side discipline + 10 protocol refinements

### Added
- **§0.4** Standing rule: every memory issue MUST trigger a refinement proposal in the same response (DEC-076).
- **§1.10** Verify-before-respond on user completeness challenge — stop, re-grep source verbatim, only respond AFTER verifying (DEC-077).
- **§4.10** Ingestion completeness discipline — forbid sample-skipping (`sed -n 'A,Bp;C,Dp'`, head/tail-only, modulus decimation); mandate sequential walk + high-water mark + coverage ≥0.99 OR `intentional_summary: true` with `summary_reason` (DEC-078).
- **§4.11** Token-budget transparency — declare chunking plan + confirm coverage in response for any source >500 lines or >50 KB (DEC-079).
- **§8.6** Source-coverage validator as Auto-Dream Phase 6 — re-hash sources, emit `op:drift_candidate` on hash mismatch, `op:shallow_candidate` on <0.80 coverage (DEC-081).
- **§3** layout extended: `memories/drift/` (auto-generated by §8.6) and `memories/refinements/` (REF-NNN-<slug>.md per adopted protocol amendment) as first-class memory bucket types (DEC-084).
- **§5.1** frontmatter additions (24 → 27 permitted fields):
  - `source_freshness_tier: <int ≥ 1 | null>` — lower = more authoritative; resolved per project from `manifest.source_tiers` (DEC-080).
  - `ingestion_coverage: <block | null>` — MANDATORY when `provenance.source ∈ {imported, doc, chat}`; carries `source_path`, `source_sha256`, `source_lines`, `processed_lines`, `source_messages`, `processed_messages`, `first_ts`, `last_ts`, `intentional_summary`, `summary_reason` (DEC-078).
  - `summary_reason: <string | null>` — required when `intentional_summary: true` (DEC-078).
- **§6** manifest additions:
  - `source_tiers: [{pattern, tier, rationale}, …]` — scope-pattern-glob → tier-int mapping for §9.1 Step 0 conflict resolution (DEC-080).
- **§7.1** audit row additions:
  - `correction_to: <evt_… | null>` — set when an op corrects the agent's own prior action (vs. a fact in the world) (DEC-083).
- **§14** end-of-response block additions:
  - Mandatory coverage suffix on any ingestion-op line (e.g. `created — coverage 944/944 lines, 53/53 messages, 2026-04-22→2026-05-04`).
  - New `drift candidates: <N>` and `shallow candidates: <N>` lines reporting §8.6 detections from the most recent consolidation (DEC-085).

### Changed
- **§9.1** Conflict decision tree gains a **Step 0** before the classification check: lower-tier (more authoritative) memory wins automatically; the higher-tier is auto-marked `superseded_by`. Step 0 is skipped when either side is `personnel` or `client` classification — those still go to manual resolution per Step 1. Eliminates Notion-vs-chat round-trip questions (DEC-080).
- **§10** Read protocol: added glances at `memories/drift/` (when the request touches a topic with multiple sources of truth) and `memories/refinements/` (when starting any substantive task — agents learn from past failure modes).

### Real-world trigger
Corrective re-ingestion of the 944-line Stephen↔Miguel WhatsApp DM. The original digest was produced via `sed -n 'A,Bp;C,Dp;…'` sampling and shipped at ~25% line coverage. Stephen surfaced the gap with screenshots and the prompt *"is your BRAIN not saved?"*. Re-ingestion captured 12 missed frozen decisions including 80/10/10, Master Seed Mirage Day-1 lock, SRF Bridge rejection, Resolution Waiting List, Vesting/Dual-Wallet, Specialization Ladder, Power Tens, Atomic Split, Failure Protection, Founder's Draw, contract-sign clock, Closed Beta MVP scope. Five of the §0.4 / §1.10 / §4.10 / §4.11 / §8.6 / §14 amendments are direct read-side counterparts to existing write-side gates (§4.1–§4.4) — the failure exposed an asymmetry in the protocol that this changelog entry closes.

## 2026-05-04 (afternoon revisions)

### Removed
- **§6 manifest** — `compatible_runtimes` field. Vestigial; not referenced anywhere in protocol logic.
- **§6 manifest** — `schema_version` field. Conceptually misaligned with the day-by-day protocol-evolution model.

### Changed
- **§4.3 file-content hygiene** — forward-compat sentence rewritten: unknown frontmatter fields now rejected with `op:rejected reason:unknown-frontmatter-field:<name>` and surfaced (was: "forward compat via manifest.schema_version").
- **§13.0 state classifier** — `INCOMPATIBLE:<sv>` row replaced with `INCOMPATIBLE:<field>`. Triggered by manifest carrying any field not in the agent's loaded §6 schema (field-presence tripwire). Same "refuse to operate; surface to user" action; the comparison just becomes structural rather than version-numbered.

### Real-world trigger
Stephen asked "is `compatible_runtimes` and `schema_version` necessary?" — neither survived the analysis. `compatible_runtimes` was unused vestigial code; `schema_version`'s discrete-version model contradicts day-by-day protocol evolution (would either bump daily and trigger constant `INCOMPATIBLE` cross-machine, or never bump and lie). Replaced with field-presence detection at the validator level, which achieves the same forward-compat protection without inline version markers.

## 2026-05-04 (afternoon revisions, follow-up)

### Changed
- **§6 manifest example** — `source_tiers` array stripped of Styx-specific patterns (`module:whatsapp-*-dm`, `module:whatsapp-*-group`, `module:notion-*`). Replaced with generic schema-only example (`<scope-glob>` + default `*` tier 99). The field is universal protocol; the values are per-project. Each project's `manifest.json` configures its own patterns at bootstrap. A new clarifying sentence after §6 makes this explicit.

### Real-world trigger
Stephen flagged that the previously-checked-in §6 example carried Styx project context (whatsapp + notion patterns), which is a correctness bug for any project that adopts AGENTS.md as its protocol — the patterns would be meaningless in cyberos or any other project. Stripping fixes the protocol's universality and aligns with the no-project-specific-examples-in-universal-docs principle (now also captured as a feedback memory).
