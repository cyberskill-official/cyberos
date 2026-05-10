# AGENTS-CORE.md — Tight normative subset (aggressive mode)

> **GENERATED FILE.** Regenerate via:
> `python3 runtime/tools/extract_agents_core.py --aggressive docs/CyberOS-AGENTS.md > docs/CyberOS-AGENTS-CORE.md`
>
> ~10K-token normative subset; load every session. The full
> `docs/CyberOS-AGENTS.md` is canonical; this file is a derived view.

---

## ⚠️ When you MUST load the full AGENTS.md

The agent MUST load `docs/CyberOS-AGENTS.md` (the full canonical doc) BEFORE doing any of the following. CORE is insufficient for these operations:

- **Any §0.5 protocol upgrade** — full §0.5 carries the canonical-form spec, signing-key TOFU rules, three-way conflict resolution, and the post-upgrade scan trigger
- **Entering MAINTENANCE mode** — full §8.8 lists the permitted/forbidden ops + auto-expiry semantics + the `maintenance_session_id` provenance contract
- **Bootstrapping a new store** — full §13.1 carries the 13-step bootstrap sequence + §0.1 forbidden-paths sanity check
- **Export or import** — full §11 covers determinism rules (§11.2), signing (§11.3), round-trip property (§11.4), single-bundle import collisions (§11.5), multi-bundle merge (§11.6), filesystem portability (§11.7)
- **Any audit-row write** — full §7.1 carries the complete row schema including all optional fields; §7.2 carries the RFC 8785 JCS canonical-JSON algorithm
- **Frontmatter validation beyond required fields** — full §5.2 carries the regex/range validators for every field type (UUIDv7/ULID, ISO-8601, confidence range, tag pattern, etc.)
- **Encryption operations (Stage 5)** — full §5.6.1–5.6.5 carries the envelope format, key derivation pipeline, Shamir 3-of-5 escrow rules, indexability constraints, audit-chain compatibility
- **Ledger compaction (Stage 6)** — full §7.7 + §8.9 cover pre-conditions, atomic phase steps, archive format, decompaction reverse path
- **Reconciliation (§4.7)** — full §4.7 covers the stale-checkpoint fallback, orphan session.start detection, orphan manifest update detection
- **Content-gate validation (§4.2)** — full §4.2 carries the complete injection-marker regex set + letters-collapsed forms + UTS #39 mixed-script rules
- **Path-traversal guard (§4.1)** — full §4.1 carries the 5-step ordered validation including Windows-portability checks
- **§0.4 refinement-proposal flow** — full §0.4 carries the 4-format refinement proposal structure + tier classification
- **§0.6 related-files update rule** — full §0.6 lists the 8-step order-of-operations for any successful op:protocol_upgrade
- **Verbose / debug / maintenance §14 mode** — full §14.2 carries the full-format end-of-response block schema

**If you are uncertain whether CORE covers your operation, default to loading the full doc.** CORE is fast-load convenience, not the canonical contract.

---

### 0.1 Real-filesystem-only memory location (sev-0)

`.cyberos-memory/` MUST be created and operated on at the **real local-filesystem path of the user's project root** — the same path the user sees in Finder / Explorer / their shell. The agent MUST refuse to bootstrap or write to any other location.

Resolution rule: `<root>` is the deepest ancestor of the user's working file that contains `.git/`, `package.json`, `pyproject.toml`, `Cargo.toml`, `go.mod`, `pom.xml`, or an `AGENTS.md`/`CLAUDE.md` placed by the user. The agent then runs `realpath`/`os.path.realpath` on `<root>` (resolving every symlink) and uses the result. The memory root is exactly `<realpath(<root>)>/.cyberos-memory/`.

**Forbidden memory locations** (case-insensitive substring match on the *resolved* absolute path; reject with `op:"rejected" reason:"virtual-fs-memory-location:<which>"` and surface to the user):

- `/sessions/`, `/private/var/folders/`, `/var/folders/`, `/tmp/`, `/private/tmp/`, `/dev/shm/`, any `tmpfs`-mounted dir
- any path containing `local-agent-mode-sessions`, `claude-hostloop-plugins`, `cowork-session`, `cowork-mode-sessions`, `agent-sandbox`, `mcp-sandbox`, `claude-code-sandbox`
- any host-provided ephemeral working dir: `outputs/`, `uploads/`, `scratchpad/`, `workspace-tmp/`, `agent-workspace/`
- any FUSE / overlay / bind-mount that does not survive a host reboot, including `<root>/mnt/`, `<root>/.sandbox/`, `<root>/.cowork/`
- network mounts (`smb://`, `afp://`, `nfs:`) unless the user has explicitly opted in for that exact path in the current chat turn

**Sanity check on every session start:**

1. Resolve `<root>` per the rule above.
2. Confirm `<realpath(<root>)>` does NOT match any forbidden pattern. If it does → stop, tell the user the agent appears to be running against a sandbox/mirror of the project rather than the real folder, and ask them to grant access to the real path.
3. Read `<root>/.cyberos-memory/manifest.json` if present and confirm `project.root_path == <realpath(<root>)>`. Mismatch → `op:"rejected" reason:"root-path-drift"`; freeze writes; surface. This is the trip-wire that catches an agent silently operating on a sandbox copy.

If the only writable filesystem the agent can reach is one of the forbidden locations, **do not** silently fall back to it. Stop, tell the user the agent cannot reach the real project folder, and either (a) ask them to grant filesystem access, or (b) ask them to manually create `<root>/.cyberos-memory/` and re-run, or (c) accept a read-only session in which no audit rows are appended and no memory is written.

`manifest.json` MUST carry `project.root_path` set to the real absolute path (§6). It is not optional.

### 0.5 Protocol update policy (sev-0)

This protocol document (`AGENTS.md`) is content-addressable: identified by the SHA-256 of its canonical form, not by an inline version marker. The manifest pins the SHA of the currently-approved protocol; every session start verifies the pin; protocol changes happen only through user-consented upgrades.

**Canonical form for SHA computation.** `sha256_hex` of the document after: NFC Unicode normalisation, BOM stripped (start and mid-file), `\r\n` and lone `\r` collapsed to `\n`, trailing whitespace trimmed per line, trailing empty lines removed, single terminating `\n` appended. The result is a deterministic byte stream that two agents on different platforms hash identically.

**Pin storage.** `manifest.json` carries (per §6 extension):

```json
"protocol": {
  "sha256": "sha256:<canonical hash of currently-approved AGENTS.md>",
  "approved_at": "<ISO-8601>",
  "approved_by": "subject:<user> | human:<role>",
  "loaded_path": "<repo-relative path; default docs/CyberOS-AGENTS.md>",
  "signing_keys": [{"fingerprint": "ed25519:<fp>", "label": "<…>", "added_at": "<ISO>", "added_by": "<actor>"}],
  "last_checked_at": "<ISO|null>"
}
```

**Session-start tripwire.** After §0.1 root resolution, the agent computes the canonical SHA of `<root>/<manifest.protocol.loaded_path>` and compares to `manifest.protocol.sha256`. Mismatch → §13.0 returns `INCOMPATIBLE:protocol-sha256-mismatch`; the agent refuses to operate, surfaces the diff, waits for human approval. Match → proceed.

**Approval phrase.** The user adopts a new version by saying *"approve protocol upgrade to `<sha256:…>`"* in the current chat turn (per §0.2 — chat-turn approval is the only authoritative source). The phrase must include the exact target SHA so silent substitutions are impossible. The agent then:

1. Copies the prior `AGENTS.md` to `meta/protocol-history/AGENTS-<before_sha256>.md` (verbatim file copy; exempt from §5.1 frontmatter — these are protocol-doc archives, not memories; integrity verified solely by SHA suffix matching content).
2. `str_replace` on `manifest.json` to update `manifest.protocol.sha256`, `approved_at`, `approved_by`.
3. Appends `op:"protocol_upgrade"` to the audit ledger with `path: ".cyberos-memory/manifest.json"`, `before_hash`/`after_hash` for the manifest, and `reason` carrying the protocol-doc SHA transition (`"<before_sha256> → <after_sha256> per §0.5; approved by <actor> in chat"`).
4. **Auto-triggers a §8.7 self-audit pass** as the post-upgrade migration check. Schema validate (§8.7 phase 1) catches memories failing the new §5.1; supersedes-graph integrity (phase 2) catches dangling relationships if scopes were renamed; resource caps (phase 6) catches new field additions pushing files over §5.5 limits. Findings surface per §8.7 severity routing — `CRITICAL` freezes writes; `WARN` appears in the §14 block; `INFO` is logged. The user reviews and approves migrations per-finding (or in batch via MAINTENANCE mode for repair ops). The auto-triggered scan is named `<YYYY-MM-DD>-<sha>-postupgrade.md` to distinguish from routine on-demand scans. Skip only with explicit phrase *"skip post-upgrade scan"* (logged as `op:"skipped-by-user"` with reason citing §0.5 step 4).

**Rollback.** User says *"rollback protocol to `<sha256:…>`"*. The agent verifies the SHA exists under `meta/protocol-history/AGENTS-<sha>.md`, runs §4.7 reconciliation against the older rules, copies that archive over `<loaded_path>`, updates the manifest pin, appends `op:"protocol_rollback"` to the audit ledger.

**Signed upstream releases (TOFU).** When `manifest.protocol.signing_keys` is non-empty AND the user runs *"check for protocol updates"*, the agent calls the org BRAIN's `protocol.releases.list` MCP tool (or fetches a known release-feed URL), retrieves `[{sha256, release_ts, signature, changelog_url}]`, verifies each release's Ed25519 signature against any pinned fingerprint, and presents the diff for approval per the phrase above. Trust establishment is TOFU: the first fingerprint enters the manifest via explicit user paste from any trusted out-of-band source — a CyberSkill-signed announcement, a verified org-wide secrets manager, an in-person fingerprint exchange, or any equivalent. **Pre-BRAIN-module-P1, no canonical out-of-band source is mandated by this protocol** (the canonical mechanism lands when P1 ships). Silent or scheduled checks are permitted; silent or scheduled APPLY is forbidden — apply always requires the chat-turn approval phrase.

**Three-way conflict (loaded ≠ pinned ≠ upstream).** When `protocol.releases.list` returns an upstream release with SHA `Z` AND `manifest.protocol.sha256 == X` AND `canonical_sha(loaded AGENTS.md) == Y` AND `X ≠ Y ≠ Z`, the agent enters a three-way conflict state. It refuses to apply upstream and surfaces a structured prompt:

```
⚠️  Three-way protocol conflict detected:
  pinned:   sha256:Xxxxx…
  loaded:   sha256:Yyyyy…  (you hand-edited; never approved)
  upstream: sha256:Zzzzz…  (signed by ed25519:fp…, dated …)

Options:
  1. "revert local protocol" → restore loaded to pinned (sha256:X…), then re-check upstream
  2. "approve protocol upgrade to sha256:Y…" → bring pinned up to your local edit
  3. Run a manual three-way merge of X/Y/Z, then approve the merged SHA via the §0.5 phrase
```

No automated merge. The agent waits for an explicit user choice in chat. This prevents silent loss of local edits to upstream pulls and prevents incompatible mixed states. The state classifier (§13.0) returns `INCOMPATIBLE:three-way-protocol-conflict` until the user chooses an option; writes are frozen during this state, identical to the 2-way `protocol-sha256-mismatch` handling.

**Bootstrap behaviour.** First-run bootstrap (§13.1) auto-pins the canonical SHA of whatever `AGENTS.md` is loaded at first run. The user is not prompted; the first run is a quiet baseline. `signing_keys` is initialised empty; the user adds CyberSkill's fingerprint later.

**Forbidden by §0.2.** Mutating `manifest.protocol.sha256` outside the chat-turn approval phrase is forbidden by §0.2.

## 1. Standing directive (every session)

1. Resolve project root **on the real local filesystem** per §0.1 (nearest `.git/`, `package.json`, `pyproject.toml`, `Cargo.toml`, `go.mod`, `pom.xml`, or `AGENTS.md`/`CLAUDE.md`). Run the §0.1 sanity check; never operate against a sandbox / mounted / temporary path.
2. Classify state of `<root>/.cyberos-memory/` per §13.0. **Auto-bootstrap silently if `PRISTINE`** (run §13.1 without prompting the user) — UNLESS the user's current-turn message contains a recognised diagnostic verb (default list: `healthcheck`, `status`, `inspect`, `audit`, `check BRAIN`, `show BRAIN`, `view BRAIN`, plus the configured `manifest.health_check_policy.on_demand_phrase` if present; project may override via `manifest.health_check_policy.diagnostic_verbs[]`). The verbs that mention "BRAIN" use uppercase per §0.3 (case-sensitive alias); lowercase "brain" verbs are NOT diagnostic triggers (they're treated as anatomy/metaphor per §0.3). When intent is diagnostic AND state is `PRISTINE`, the agent enters `PRISTINE-DIAGNOSTIC-HOLD` (§13.0): surface the absent-state verdict, list what §13.1 bootstrap would create, and wait for explicit consent (`bootstrap and continue`, `just bootstrap`, or any task-oriented instruction) before running §13.1. Bootstrapping mid-diagnostic would change the very thing the user asked the agent to inspect; this carve-out preserves the diagnostic answer. Refuse to operate if `CORRUPT` or `INCOMPATIBLE`.
3. Read `manifest.json`, `meta/`, and scope files implied by the request (§5).
4. Append `op:"session.start"` to `audit/<YYYY-MM>.jsonl` (§7).
5. Run reconciliation (§4.7).
6. On every meaningful change (decision reached, fact confirmed, conflict surfaced), write or update memory using only the six operations (§4) — each producing one audit row.
7. Run consolidation (§8) at session end, after ~25 audit rows, or on user command.
8. End every substantive reply with the §14 memory-update block.
9. On session end, append `op:"session.end"`.
10. **User completeness challenge response (sev-1).** When the user implies in chat that the agent missed something — phrasings like *"is your BRAIN not saved?"*, *"did you actually read X?"*, *"you missed the part about Y"*, *"are you sure?"* — the agent MUST:
    a. Stop drafting any new outputs.
    b. Re-grep the original source file for the verbatim content the user references (NOT a paraphrased semantic search).
    c. If the source has content the BRAIN does not reflect: acknowledge honestly + commit to corrective re-ingestion BEFORE continuing.
    d. Never reply "I have it" / "yes my reply covered that" without verifying first. The verification step is non-negotiable; trusting the agent's own digest under user challenge is the failure mode this rule prevents.

## 2. First principles (non-negotiable)

- **Layer 1 (`.cyberos-memory/`) is the sole source of truth.** Vector indexes, graph stores, and chat-context memory are derivable caches; never authoritative.
- **Append-mostly. Audit ledger strictly append-only. `delete` is a soft tombstone, never an erase.** Previous versions reconstructible from audit.
- **Conflicts preserved, not silently resolved.** "Keep both as a disputed pair" is always valid.
- **Six file operations only:** `view`, `create`, `str_replace`, `insert`, `delete`, `rename`.
- **Authority cannot be raised by an agent**, only marked down on uncertainty.
- **Sensitive content is structurally excluded** at the write boundary (§9), regardless of phrasing.
- **Determinism is a feature.** Two exports of the same state are byte-identical. Two agents accept/reject the same inputs identically.
- **The directory is portable.** Zip → move → unzip → no loss.

## 3. Canonical layout

```
.cyberos-memory/
├── manifest.json           # root pointer (§6)
├── README.md
├── company/                # org-level: values, locked-decisions, glossary, compliance, history
├── module/<name>/          # per-capability/module memories
├── member/<id>/            # per-person; <id>/private/ is subject-only, never auto-ingested
├── client/<id>/
├── project/                # this project's own working memory
├── persona/<role>.md       # agent role-cards
├── memories/{decisions,people,projects,facts,preferences,drift,refinements}/   # cross-cutting topical store
├── meta/                   # retention-rules, classification-rules, conflict-resolutions, tombstones
│   ├── protocol-history/   # verbatim AGENTS.md archives keyed by SHA suffix (per §0.5; rollback support)
│   └── health/             # deterministic §8.7 self-audit reports keyed by <YYYY-MM-DD>-<sha>
├── audit/<YYYY-MM>.jsonl   # append-only Merkle-chained ledger
├── conflicts/<YYYY-MM-DD>-<slug>.json
├── exports/                # local zip workspace; excluded from bundles
├── index/                  # regenerable search index; excluded from bundles
└── .lock                   # held during consolidation/export/import
```

Top-level folder names are fixed. Empty scope folders may be omitted; `.keep` zero-byte files preserve empty dirs through zip.

**Filenames** in `memories/<bucket>/`: `<TYPE>-<NNN>-<slug>.md` (e.g., `DEC-007-pricing-tiers.md`). Slugs lowercase-kebab. Numeric prefix monotonic per directory across lifetime — `next = max(seen)+1`, including tombstoned files. Numbers never reused.

## 4. The six operations

| Op            | Use                                               | Forbidden                                   |
| ------------- | ------------------------------------------------- | ------------------------------------------- |
| `view`        | Read file or list dir; paginate at 10 KB          | binary streams                              |
| `create`      | New file, full content; fail if path exists       | overwrite                                    |
| `str_replace` | Replace exactly one substring                     | multi-region edits in one call              |
| `insert`      | Append/insert at line N                           | non-additive edits                          |
| `delete`      | Soft-delete (tombstone, §4.6)                     | hard erase; touching `audit/`               |
| `rename`      | Move file/dir **within the same scope** (intra-scope only); allowed e.g. moving to `<scope>/private/` | renaming `audit/`, `manifest.json`, `.lock`, or **across scope boundaries** (e.g., `member/<x>/` → `client/<y>/`) |

`view` produces no audit row (use a separate `retrieval-<YYYY-MM>.jsonl` if sensitive-read auditing is required). Every other op produces exactly one audit row.

### 5.1 Frontmatter schema (closed set; 28 base fields + Stage 5 encryption block)

The schema below lists 28 base fields. Stage 5 (sha256:d3ce97…) added two encryption-envelope fields (`encrypted: bool`, `encryption: {algorithm, nonce, aad}`) that apply only when `manifest.encryption_policy.enabled = true` per §5.6. The closed-set rule applies to the union of these; new fields beyond this union require §0.5 protocol upgrade.

```yaml
---
memory_id:        mem_<UUIDv7-or-ULID>
scope:            company | meta | module:<name> | member:<id> | client:<id> | project:<slug> | persona:<role> | dm:<a>:<b>
classification:   personnel | client | operational | public
authority:        human-edited | human-confirmed | llm-explicit | llm-implicit
version:          <int ≥ 1>
created_at:       <ISO-8601 with offset>
created_by:       subject:<id> | human:<role> | agent:<name> | system:<service>
last_updated_at:  <ISO-8601 with offset>
updated_by:       <same forms as created_by>
supersedes:       <memory_id | array of memory_ids | null>
superseded_by:    <memory_id | null>
expires_at:       <ISO-8601 with offset | null>
provenance:
  source:         chat | doc | code | inference | manual | imported | conflict_resolution
  source_ref:     <opaque ref, 1–2048 UTF-8 bytes>
  confidence:     <float in [0.0, 1.0]>
consent:
  has_consent:    <bool — must be true for personnel/client>
  consent_event:  <opaque id | null>
  consent_scope:  [<lower-snake/colon strings>, ≤16; non-empty for personnel/client]
tags:             [<kebab>, ≤32, no duplicates]
relationships:    [{relates_to: <mem_…>, kind: refines|contradicts|depends-on|derives-from|summarises|cites}, ≤64]
retention:
  rule:           <name from meta/retention-rules.md>
  earliest_delete: <ISO date | null>
embedding:        {model: <string|null>, version: <string|null>, vector_id: <string|null>}
sync_class:       local-only | publishable | shared | client-visible    # see §17; defaults from scope per §17 unless overridden per-file
source_freshness_tier: <int ≥ 1 | null>   # lower = more authoritative; resolved per project from manifest.source_tiers (§6). null = use default tier 99 (lowest priority).
ingestion_coverage:                         # MANDATORY for any memory with provenance.source ∈ {imported, doc, chat}
  source_path: <abs path or canonical opaque ref>
  source_sha256: <sha256:…>
  source_lines: <int>
  processed_lines: <int>
  source_messages: <int|null>
  processed_messages: <int|null>
  first_ts: <iso8601|null>
  last_ts: <iso8601|null>
  intentional_summary: <bool>
  summary_reason: <string|null>             # required when intentional_summary == true
signed_by:        <ed25519:<base64sig> | null>
# (when tombstoned)
tombstoned:       true
deleted_at:       <ISO-8601>
deleted_by:       <actor>
tombstone_reason: <string>
---

# H1 title

Body in plain Markdown (≤ 10 KB ideal, 30 KB hard). End with `## How to use this memory` (1–5 sentences for the next agent) and `## History` (dated bullet list).
```

**Frontmatter compactness (write-side).** When emitting frontmatter, omit any field whose value is `null` OR an empty array OR an empty object, EXCEPT for fields explicitly required by `classification` (consent block for `personnel`/`client`) or `tombstoned: true` (deleted_at/deleted_by/tombstone_reason). Read-side accepts both compact and verbose forms — omitted optional fields default to `null`/empty. The 28-field closed-set rule applies only to *recognised* fields; absence of optional fields is not a schema violation.

### 5.4 Classification → retention

| Classification | Default retention | Consent rule                                           |
| -------------- | ----------------- | ------------------------------------------------------ |
| `personnel`    | 3 years            | Subject consent required. Subject can edit/delete own at will. |
| `client`       | 7 years            | Active engagement OR explicit consent.                  |
| `operational`  | 1 year             | None.                                                  |
| `public`       | indefinite         | None.                                                  |

### 5.5 Resource caps (hard unless noted)

| Limit                              | Value                                                          |
| ---------------------------------- | -------------------------------------------------------------- |
| Per-file body                      | 10 KB ideal, 30 KB hard                                         |
| Per-file frontmatter               | 4 KB                                                           |
| Per-store total size               | 1 MB soft / 10 MB hard (consolidate first if approaching)       |
| Per-store file count               | 10 000                                                          |
| Per-store directory depth          | 12 from `.cyberos-memory/`                                       |
| Audit row serialised               | 64 KB                                                           |
| Audit row `diff` field             | 2 KB; longer → store `"<hash-only>"` plus `after_hash`           |

## 6. `manifest.json`

```json
{
  "memory_layer": 1,
  "tenant":  {"id": "<slug>", "name": "<display>", "residency": "<region>"},
  "owner":   {"kind": "human", "id": "<slug>", "display_name": "<display>"},
  "project": {"id": "prj_<slug>", "name": "<display>", "root_path": "<absolute>", "language": "<en|vi|...|mixed>", "stack": ["<detected>"]},
  "scope_root": "project:prj_<slug>",
  "timezone": "<IANA timezone>",
  "created_at": "<ISO-8601>",
  "last_updated_at": "<ISO-8601>",
  "memory_count": 0,
  "audit_chain_head": "sha256:<64 hex>",
  "exclusion_rules": [
    {"kind": "regex", "pattern": "(?i)\\b(salary|payslip|bonus|equity|grant)\\b",   "reason": "compensation"},
    {"kind": "regex", "pattern": "\\b(?:passport|national_id|ssn|tax_id)\\b",       "reason": "gov-id"},
    {"kind": "regex", "pattern": "\\b(?:iban|swift|account_number)\\b",             "reason": "bank"},
    {"kind": "topic", "value": "personal_life:health",                              "reason": "special-category PII"},
    {"kind": "topic", "value": "secrets:api_keys",                                  "reason": "secrets"}
  ],
  "scope_contract": {
    "agent_default_write_scopes": ["project", "meta", "module"],
    "agent_default_read_scopes":  ["company","module","member","client","project","persona","meta"],
    "elevated_scopes_require_human_confirmation": ["company","member","client","persona"]
  },
  "size_limits": {"per_file_body_kb": 10, "per_file_hard_kb": 30, "per_tenant_total_mb": 10},
  "languages": ["en"],
  "language_routing_default": "en",
  "signing_key_fingerprint": null,
  "protocol": {
    "sha256": "sha256:<canonical hash of currently-approved AGENTS.md>",
    "approved_at": "<ISO-8601>",
    "approved_by": "subject:<user>",
    "loaded_path": "docs/CyberOS-AGENTS.md",
    "signing_keys": [],
    "last_checked_at": null
  },
  "operational_mode": "normal",
  "health_check_policy": {
    "on_session_end": true,
    "on_demand_phrase": "run BRAIN healthcheck",
    "post_upgrade_phrase": "rescan BRAIN",
    "diagnostic_verbs": ["healthcheck", "status", "inspect", "audit", "check BRAIN", "show BRAIN", "view BRAIN"]
  },
  "source_tiers": [
    {"pattern": "<scope-glob>", "tier": <int ≥ 1>, "rationale": "<why this scope is more authoritative than others>"},
    {"pattern": "*", "tier": 99, "rationale": "Default — lowest priority unless overridden."}
  ],
  "reconciliation_checkpoint": {
    "audit_id": "<evt_…|null>",
    "chain": "<sha256:…|null>",
    "ts": "<ISO-8601|null>"
  },
  "read_profile": {
    "eager_scopes": ["meta"],
    "lazy_scopes": ["company", "module", "member", "client", "project", "persona", "memories"]
  },
  "encryption_policy": {
    "enabled": false,
    "scopes": ["member:<self>/private", "classification:personnel", "classification:client"],
    "algorithm": "xchacha20poly1305-ietf",
    "key_derivation": "hkdf-sha256-from-hardware-bound",
    "fallback_kdf": "argon2id-t3-m64-p4",
    "passphrase_strength_minimum": {"min_chars": 16, "zxcvbn_score": 3}
  },
  "shamir_fragments": {
    "threshold": 3,
    "total": 5,
    "master_key_fingerprint": null,
    "fragments": []
  }
}
```

**`reconciliation_checkpoint`** records the most recent successfully-completed `op:"session.end"` or `op:"consolidation_run"` row. §4.7 reconciliation walks only rows after this checkpoint when present; falls back to full walk on missing/stale (>30 days) checkpoints or any chain-mismatch. Updated atomically with `op:"session.end"` and `op:"consolidation_run"` writes; never edited independently.

**`read_profile`** declares which scopes load eagerly vs on-demand at session start. Default profile shown above; projects may override. See §10.

**`encryption_policy`** — opt-in at-rest encryption per §5.6. Default `enabled: false`. Mutating any field requires the wizard flow at `runtime/tools/cyberos_encrypt.py enable` or chat-turn approval per §0.5. The `scopes` list uses the syntax `<scope-pattern>` for paths or `classification:<class>` for classification-keyed selection. Memories matching ANY entry are encrypted.

**`shamir_fragments`** — recovery-escrow registry per §5.6.3. Default empty array. Fragments themselves are NEVER stored here — only their fingerprints. Threshold and total are pinned at enable time and rotated only via `op:"shamir_rotation"`. Each entry in `fragments` is `{label, fingerprint, created_at, distributed_at|null}`.

**`source_tiers`** is per-project: each project configures its own scope-pattern globs and tier integers based on which sources are most authoritative for that project's domain. The example above shows only the schema and the default tier — projects fill in their actual scope patterns at bootstrap time. Patterns are matched greedy/most-specific-wins; ties resolve by array order. A scope with no matching pattern receives tier 99 (lowest priority).

`audit_chain_head` is a **witnessed checkpoint** — the `chain` value the ledger held at the moment of the most recent manifest update. It will normally lag the ledger end by 1+ rows. Validators walk the ledger end-to-end for chain integrity AND confirm `audit_chain_head` appears in the ledger.

All `manifest.json` mutations go through `str_replace` (so they hit the audit log).

### 7.1 Row schema

```json
{
  "audit_id": "evt_<UUIDv7-or-ULID>",
  "ts": "<ISO-8601>",
  "actor_kind": "agent|human|system|subject",
  "actor_id": "<actor>",
  "persona": "<persona|null>",
  "op": "session.start|session.end|create|str_replace|insert|delete|rename|view|rejected|revert|corrects|consolidation_run|export|import|skipped-by-user|lock_recovered|protocol_upgrade|protocol_rollback|health_check|warn|drift_candidate|shallow_candidate|maintenance.start|maintenance.end|ledger_compact|ledger_decompact|encryption_policy_change|key_rotation|key_recovery_initiated|key_recovered|shamir_rotation|shamir_distribution_confirmed",
  "scope": "<scope>",
  "path": ".cyberos-memory/<rel>",
  "memory_id": "<mem_…|null>",
  "prev_version": <int|null>,
  "new_version": <int|null>,
  "supersedes_event_id": "<evt_…|null>",
  "classification": "<class|null>",
  "authority": "<auth|null>",
  "consent_event_id": "<id|null>",
  "provenance": {"source": "<…>", "source_ref": "<…>", "confidence": <float>},
  "before_hash": "<sha256:…|null>",
  "after_hash": "<sha256:…|null>",
  "diff": "<unified-diff or '<hash-only>'>",
  "reason": "<≤200 chars present-tense citing source>",
  "correction_to": "<evt_…|null>",

  "prev_chain": "<sha256:… | sha256:0…0 (genesis)>",
  "chain": "<sha256:…>"
}
```

**`correction_to` semantics.** When an op corrects the agent's *own prior action* (vs. corrects a fact in the world), `correction_to` MUST be set to the prior `audit_id` being corrected. This distinguishes "the agent fixed its own mistake" from "the user changed their mind." Future agents reading the chain can de-emphasise corrected rows in retrieval and audit trails. `correction_to` for fact-correcting ops (the user supplied a new fact superseding the old) stays `null`; `op:"corrects"` is used instead.

### 7.2 Canonical JSON for hashing (deterministic; RFC 8785 JCS)

`chain = sha256_hex(canonical_json(row_without_chain_or_prev_chain) || prev_chain)`, prefixed `sha256:`. The body passed to `canonical_json` MUST exclude both the `chain` and `prev_chain` keys; `prev_chain` is concatenated as raw bytes after the canonical body.

**Canonicalisation algorithm: RFC 8785 JCS (JSON Canonicalization Scheme).** Implementations MUST conform to RFC 8785 in full. The summary below is normative; where this protocol's wording differs from RFC 8785, RFC 8785 wins.

- **Object key ordering**: lexicographic on UTF-16 code units (RFC 8785 §3.2.3). For ASCII-only keys this is equivalent to byte-lex order. For keys containing non-BMP characters, the UTF-16 code-unit order matters.
- **Whitespace**: none. No spaces between separators; no leading/trailing whitespace anywhere; no trailing newline at end of canonical output.
- **Separators**: `,` between array items and object members; `:` between object key and value. Exactly these single ASCII bytes; never with surrounding whitespace.
- **Strings**: UTF-8 encoded; non-ASCII preserved verbatim (NOT `\uXXXX`-escaped); the only required escapes are `\"`, `\\`, `\b`, `\f`, `\n`, `\r`, `\t`, and `\u00XX` for control chars `U+0000`–`U+001F`. Implementations MUST emit the **shorter** form where the spec permits a choice (e.g., `\n` over `
`). Strings MUST be NFC-normalised before serialisation.
- **Numbers**: ECMAScript `Number.prototype.toString` (RFC 8785 §3.2.2.3) — i.e., the shortest decimal that round-trips through IEEE-754 double-precision. Concretely: integers serialise without trailing `.0` (e.g., `1` not `1.0`); fractional values use the shortest representation (`0.7` not `0.69999…`); the literal `1.0` from a Python `float` MUST serialise as `1`, NOT `1.0`. **This is the single most common cross-writer-version divergence — implementations MUST validate against the JCS test vectors.**
- **Booleans and null**: `true`, `false`, `null` (lowercase; the only legal forms). Python `True`/`False`/`None` MUST serialise to lowercase JCS forms.
- **Arrays**: order-preserving (JSON arrays are ordered); no canonicalisation of element order.
- **No duplicate keys** in any object.

**Reference implementations**: Python `rfc8785` package (PyPI), JavaScript `canonicalize` package (npm). Hand-rolled `json.dumps(sort_keys=True, separators=(",", ":"), ensure_ascii=False)` is approximate-but-not-bit-identical to JCS — it differs on number serialisation (Python emits `1.0`; JCS emits `1`) and on UTF-16 vs byte-level key ordering. Hand-rolled implementations MUST run the JCS test vectors and confirm bit-identical output before being trusted to chain audit rows.

**Cross-writer-version compatibility.** The `chain` LINK invariant (`row[N].prev_chain == row[N-1].chain`) is the authoritative integrity guarantee — it is preserved across writer-version changes because each writer uses ITS OWN canonical-JSON output for ITS OWN row, then the next writer reads `chain` as an opaque string. Hash *recomputation* across writer versions MAY fail (different writer outputs different bytes for the same logical row); this is informational and not a chain break. Implementations MUST verify LINK integrity and MAY report hash-recomputation diffs as `INFO`-severity findings during §8.7 self-audit.

### 8.1 Surface
Walk audit since last `consolidation_run`. Identify: explicit `remember:`/`forget:` markers, user corrections, terms repeated >3× across separate creates, decisions (`reason` contains decided/approved/chose/rejected), `relationships.kind:contradicts` pairs.

### 8.2 Detect conflicts
For every same-scope contradiction, ensure both files link `kind:contradicts` and a `conflicts/<YYYY-MM-DD>-<slug>.json` exists.

### 8.3 Conservative merge
Convert relative dates to absolute (project timezone). Auto-resolve only when both sides are `operational|public` AND new authority ≥ old. Auto-dedupe only when `tags` overlap ≥ 3 AND body trigram-Jaccard ≥ 0.8 (mark loser `superseded_by`; do NOT delete here). Never auto-resolve `personnel`/`client`.

### 8.4 Reorganise
Split files > 10 KB by extracting H2 sections into siblings + one-line link. Promote `memories/facts/` >30 entries into themed sub-dirs.

### 8.5 Update manifest
Recompute `memory_count`, `last_updated_at`, `audit_chain_head`. One `str_replace` on `manifest.json`. Append `op:"consolidation_run"`. Output a 3-line summary to user: added / merged / open-conflicts (extended in §8.6 with drift/shallow counts and §8.7 with health severity counts).

### 9.1 Conflict decision (apply in order; halt on first match)

0. **Source freshness tier.** Compare `source_freshness_tier` (from frontmatter; default 99 if null). The lower-tier (more authoritative) memory wins automatically; the higher-tier memory is auto-marked `superseded_by`. Apply BEFORE any other check, BUT skip this step (defer to step 1) if either side is `personnel` or `client` classification — those still go to manual resolution. Tier comparisons resolve drift like Notion-vs-chat without needing manual review.
1. Either side `classification ∈ {personnel, client}` → write `conflicts/<…>.json`, link both `kind:contradicts`, surface, **NEVER auto-resolve**.
2. New authority strictly > old → new wins; old gets `superseded_by`; audit `op:str_replace`.
3. New authority == old → newer `ts` wins (last-writer-wins allowed only for `operational` / `public`).
4. New authority < old → old wins; new written to `conflicts/` with reason `"lower authority"`.

UI in chat always presents 4 options: **Keep A / Keep B / Keep both as disputed pair / Edit and replace both with a new statement.**

### 9.3 Denylist (sev-0 — never write to memory)

- Compensation: salary, payslip, bonus, equity grants, RSUs, total-rewards calculations, derivable comp columns.
- Government IDs: national IDs, passport, tax ID, social-security-equivalents, driver's licence.
- Bank/card: account numbers, IBAN, SWIFT, full PANs.
- Home addresses (work addresses ok in `client/` with consent).
- Health PII / special-category (incl. health leave-reason text).
- Individual peer-review scores (aggregates ok).
- Secrets: raw API keys (`sk-…`, `pk_live_…`, etc.), `.env` contents, OAuth tokens, refresh tokens, session cookies, private keys, certificates, mnemonics, recovery phrases, DB connection strings with credentials.
- External-party PII without explicit consent (`client/<id>/` requires `has_consent: true`).

If a memory must reference a denylisted item, store a **pointer** instead (`"see <vault-name> → <folder> → <entry>; held by <person>"`). If user insists on storing the value, push back once and refuse.

**Encryption is NOT a denylist softener.** When `manifest.encryption_policy.enabled = true` (§5.6), the encryption envelope protects classification-eligible content from disk-level snooping. It does NOT change what content is allowed to be written. The denylist categories above (compensation, ESOP, gov IDs, bank/card, home addresses, health PII, secrets, external-party PII without consent) remain forbidden from ANY storage form — encrypted or plaintext. The §4.2 content gate runs BEFORE the encryption envelope; denylist hits are rejected before any cryptographic operation.

### 13.0 Classify state (read-only — never writes to disk)

| State                | Detection                                                                                                       | Action                                          |
| -------------------- | --------------------------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| `PRISTINE`           | `.cyberos-memory/` does not exist                                                                                 | §13.1 (silent), unless §1 step 2's diagnostic-verb carve-out applies → `PRISTINE-DIAGNOSTIC-HOLD` |
| `PRISTINE-DIAGNOSTIC-HOLD` | Sub-state of `PRISTINE`: `.cyberos-memory/` does not exist AND user's current-turn message contains a recognised diagnostic verb (per §1 step 2 / `manifest.health_check_policy.diagnostic_verbs[]`) | Surface absent-state verdict + list what §13.1 would create; await explicit consent in chat before bootstrapping; do NOT write to disk during this state |
| `COMPLETE_BOOTSTRAP` | Dir exists; no `manifest.json`; audit empty/absent                                                              | Resume bootstrap (idempotent: write only missing) |
| `READY`              | `manifest.json` parses; `manifest.audit_chain_head` appears as a chain in the ledger (or = genesis when empty); every field in `manifest.json` is recognised by the agent's loaded §6 schema | Proceed to read protocol                         |
| `CORRUPT:<reason>`   | Chain mismatch / audit without manifest / unparseable manifest / reconciliation failure                          | Freeze writes; emit `op:"rejected"`; surface; no auto-repair |
| `INCOMPATIBLE:<field>` | `manifest.json` carries any field not in the agent's loaded §6 schema (forward-compat tripwire — agent too old) | Refuse to operate; surface the unknown field to user; ask them to run the latest AGENTS.md |
| `INCOMPATIBLE:protocol-sha256-mismatch` | Canonical SHA of `<root>/<manifest.protocol.loaded_path>` ≠ `manifest.protocol.sha256` (per §0.5) | Refuse to operate; surface the diff; require user to either revert the AGENTS.md edit OR approve the new SHA via the §0.5 chat-turn approval phrase |
| `INCOMPATIBLE:three-way-protocol-conflict` | An upstream release SHA `Z` is available AND loaded SHA `Y` ≠ pinned SHA `X` ≠ `Z` (per §0.5 three-way-conflict subsection) | Refuse to operate; surface the three-way diff; require user to choose: revert local, approve local as upgrade, or manual merge — never auto-apply upstream |

### 14.1 Compact format (sev-2; normal mode, non-BRAIN file change, no issues)

Emit when a non-BRAIN file changed this turn AND no issues are present AND mode is normal. Format:

```
---
📁 Files changed:
- <path>: <one-line description of change>
- <path>: <one-line description of change>
[- Tokens: <N input / M output>     # only when a token counter is wired up; omitted otherwise]
```

Rules:
- **Non-BRAIN paths ONLY.** Files inside `.cyberos-memory/` are NEVER listed in §14.1 — they are agent housekeeping. The user trusts the audit ledger for that detail.
- **One bullet per file.** One-line description style. No `outside BRAIN` qualifier (the path itself signals it's outside BRAIN — there are no BRAIN paths in §14.1).
- **No `Δ Changes:` heading, no `Status:` block, no `unchanged:` line, no `audit/<YYYY-MM>.jsonl: …` line.** Stripped — `📁 Files changed:` is the only section.
- **Tokens line is optional and conditional.** Emit only when the runtime exposes a token counter via tool (e.g., a future MCP exposing `usage.input_tokens`/`usage.output_tokens`). Pre-availability, omit the line — never estimate via `tiktoken` or character count, since approximations mislead.


---

**Sections elided here** (consult full AGENTS.md for any of these):

§0.4 (refinement standing rule), §0.6 (related-files rule), §4.1–4.11 (op gates, hygiene, scope contract, tombstone, reconciliation, .lock semantics, ingestion completeness, token-budget transparency), §5.2/5.3/5.6 (validators, authority hierarchy, encryption envelope), §7.3–7.7 (JSONL parsing, forbidden ledger ops, op:corrects vs correction_to, Merkle checkpoints, ledger compaction), §8.6–8.9 (source-coverage validator, self-audit pass, MAINTENANCE mode, ledger compaction phase), §9.2/9.4–9.7 (conflict file, opt-in topics, supersedes graph, locked decisions, natural-language CRUD), §10 (read protocol), §11 (export/import), §12 (prompt-injection awareness), §13.1 (bootstrap), §14.2/14.3 (verbose §14 + coverage stat), §15 (multi-agent interop), §16 (tie-breakers), §17 (personal vs shared boundary).
