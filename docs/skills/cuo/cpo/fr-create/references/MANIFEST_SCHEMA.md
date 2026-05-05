# `fr-manifest@2` — single source of truth for fr-create state

> Sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §3 (modulo s/`fr_create_and_audit`/`fr_create`/g — see `CHANGELOG.md`).

## §3.1 Hashing — staleness detection

Three SHA-256 hashes anchor re-entrancy:

- **`requirements_hash`** — SHA-256 of the normalised concatenation of every requirements file. Normalisation per file, then concatenated in lexical path order with a `\n---FILE: <path>---\n` delimiter: convert line endings to `\n`; strip BOM; strip trailing whitespace per line; collapse ≥3 blank lines to 2; ensure single trailing `\n`. Reject (issue `bootstrap-encoding`) if any input is not valid UTF-8 (binary inputs like `.pdf` are first text-extracted; the extracted text is what is hashed).
- **`fr_hash`** — over each generated FR markdown using the same normalisation. Recomputed at every write and at the top of every WORKER invocation.
- **`audit_hash`** — copied from the chained `fr-audit` skill's report (`audited_file_sha256` field) after each audit run. Optional in `fr-create` standalone mode.

## §3.2 Re-entrancy invariants

On every invocation, AFTER emitting `CONTRACT_ECHO` and BEFORE any other action:

1. Recompute `requirements_hash` from current input files. If different from `manifest.inputs.requirements_hash`, emit `INPUTS_CHANGED` (see `FAILURE_MODES.md`), set `manifest.plan.status = INVALIDATED`, write the manifest, HALT.
2. For every FR with `status = PASS`, recompute `fr_hash` from the on-disk file. If it differs from manifest, raise an HITL issue with category `stale_fr_disposition` (rule_id `STALE-001`) carrying the affected FR ID and the before/after hashes. STALE no longer halts on its own; options `ACCEPT_EXTERNAL_EDIT` / `REVERT_TO_MANIFEST` / `MARK_ERRORED` surface in the next HITL block.
3. If `manifest.json` exists but is malformed (parse error / schema mismatch), rename to `manifest.json.corrupt-<iso8601>` if the runtime allows, otherwise embed verbatim in a `bootstrap_error` field of a fresh manifest. Record a `bootstrap` issue.
4. Initialize `manifest.amendment_stats` if absent. Recompute `ratio_last_5_batches` from the trailing 5 `batch_runs` entries. If ratio
> `ratio_threshold` (default 0.2), append a soft warning to the next
   `BATCH_RUN_LOG.md` entry: *"AMENDMENT_FREQUENCY_WARNING: <N>/5 recent batches required mid-flight amendments."* Does NOT halt.
5. Initialize `manifest.amendments_pending` to `[]` if absent. Carry-over from prior runs is preserved verbatim.

## §3.3 Schema (`<output_dir>/manifest.json`)

```json
{
  "schema_version": "fr-manifest@2",
  "skill_id": "cuo/cpo/fr-create",
  "skill_revisions": {
    "fr_create": "fr_create@2.0.0",
    "_note": "MUST match the prompt_revision literal in cuo/cpo/fr-create/SKILL.md CONTRACT_ECHO. Mismatch → CONTRACT_DRIFT (FAILURE_MODES.md). The template version (feature_request@1, loaded from cyberos/docs/contracts/feature-request/v1/template.md via depends_on_contracts:) advances lockstep with this skill — they are not separately versioned."
  },
  "inputs": {
    "requirements_files": [
      {"path": "<relative>", "media_type": "<mime>", "sha256": "<hex>"}
    ],
    "requirements_hash":    "<hex>",
    "requirements_locked_at": "<ISO 8601 UTC>"
  },
  "plan": {
    "status":           "DRAFT | AWAITING_APPROVAL | APPROVED | INVALIDATED | AMENDED_AWAITING_APPROVAL",
    "approved_at":      "<ISO | null>",
    "approved_by":      "<id | null>",
    "approval_hash":    "<hex of canonical backlog | null>",
    "backlog": [
      {
        "id":                            "FR-NNN",
        "slug":                          "kebab-case-≤60-chars",
        "title":                         "<≤72 chars>",
        "one_liner":                     "<≤140 chars>",
        "feature_type":                  "user_facing | internal_tooling | integration | infrastructure",
        "tentative_eu_ai_act_risk_class":"not_ai | minimal | limited | high | escalate",
        "tentative_client_visible":      true,
        "tentative_ai_authorship":       "none | assisted | co_authored | generated_then_reviewed",
        "priority":                      "p0 | p1 | p2 | p3",
        "depends_on":                    ["FR-001", "..."],
        "estimated_size":                "S | M | L | XL",
        "open_questions":                ["<plain text>"],
        "source_refs": [
          {"file": "<path>", "page": 7, "section": "<heading>", "quote_excerpt": "<≤120 chars, ≤15 words>"}
        ],
        "status":                        "PLANNED | DRAFTING | AUDITING | PASS | HITL_PAUSE | EXHAUSTED | ERRORED | STALE",
        "created_at":                    "<ISO 8601 UTC>"
      }
    ]
  },
  "batch_runs": [
    {
      "run_id":                "<uuid>",
      "started_at":            "<ISO>",
      "ended_at":              "<ISO | null>",
      "batch_size_requested":  3,
      "batch_size_completed":  2,
      "outcome":               "BATCH_COMPLETE | BATCH_COMPLETE_WITH_AMENDMENTS | HALTED_HITL | EXHAUSTED | ERRORED",
      "frs_touched":           ["FR-001", "..."],
      "amendment_ids":         ["AMD-NNN"]
    }
  ],
  "frs": {
    "FR-NNN": {
      "slug":              "<same as backlog>",
      "file_path":         "<output_dir>/FR-NNN-<slug>.md",
      "audit_path":        "<output_dir>/FR-NNN-<slug>.audit.md",
      "status":            "<same enum as backlog>",
      "fr_hash":           "<hex | null>",
      "audit_hash":        "<hex | null>",
      "iteration_count":   0,
      "max_iterations":    10,
      "blocking_issues": [
        {
          "id":           "ISS-NNN",
          "category":     "<one of hitl_categories>",
          "rule_id":      "<e.g. QA-001>",
          "raised_at":    "<ISO>",
          "resolved_at":  "<ISO | null>",
          "resolution":   "<HITL answer payload | null>"
        }
      ],
      "amendment_triggered": "<AMD-NNN | null>",
      "created_at":        "<ISO>",
      "last_updated_at":   "<ISO>"
    }
  },
  "hitl_pending": {
    "any_blocking":                   false,
    "consolidated_request_emitted_at":"<ISO | null>",
    "frs_paused":                     ["FR-NNN"]
  },
  "amendment_stats": {
    "batch_count":           0,
    "amendment_count":       0,
    "ratio_last_5_batches":  0.0,
    "ratio_threshold":       0.2
  },
  "amendments_pending": [
    {
      "amendment_id":    "AMD-NNN",
      "trigger_fr":      "FR-NNN",
      "proposed_change": "ADD | SPLIT | MERGE | REORDER | RECLASSIFY",
      "risk_class":      "low | medium | high",
      "proposed_diff":   "<verbatim AMENDMENT_PROTOCOL.md diff>",
      "reason":          "<2-4 sentences>",
      "raised_at":       "<ISO>",
      "resolution":      null
    }
  ]
}
```

## §3.4 Write discipline

Every state-changing step MUST flush the manifest before the next step begins. The manifest is the only authoritative state — the skill MUST NOT cache state in chat context, since context resets are expected. The manifest MUST be written with deterministic key ordering (alphabetical inside objects) and 2-space indent.

The skill MUST NOT delete entries from `plan.backlog` or `frs`. Removal of an FR from scope is recorded as `status: ERRORED` with a reason or as a `PLAN_AMENDMENT_REQUEST` (see `AMENDMENT_PROTOCOL.md`).

## BATCH_COMPLETE format

Emitted at end of WORKER phase when the batch is done (PASS / HITL_PAUSE / EXHAUSTED).

```
BATCH_COMPLETE
run_id:                  <uuid>
batch_size_requested:    <int>
batch_size_completed:    <int>
outcome:                 BATCH_COMPLETE | BATCH_COMPLETE_WITH_AMENDMENTS
frs_touched:             [FR-NNN, ...]
amendments_raised:       [AMD-NNN, ...]
amendment_stats_after:   {batch_count: <int>, amendment_count: <int>,
                          ratio_last_5_batches: <float>, threshold: <float>}
warnings_emitted:        [AMENDMENT_FREQUENCY_WARNING, ...]
next_skill_recommendation: cuo/cpo/fr-audit
next_step:               <human-readable suggestion>
```
