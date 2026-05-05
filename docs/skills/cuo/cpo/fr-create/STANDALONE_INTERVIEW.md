# `fr-create` standalone interview script

> Used by the runtime when `fr-create` is invoked **standalone** (no upstream pipeline envelope) — to fill the `expects.required_fields` from the user via chat. When invoked **chained**, this file is ignored; the upstream skill's output envelope supplies the same fields machine-readably.

## Mode detection

The skill is in standalone mode when ANY of:

- No `pipeline_run_id` is present in the call context.
- The caller is the CHAT primitive (a human typing in a thread), not another skill.
- The input envelope is empty or `{}`.

Otherwise the skill is in chained mode; this interview is skipped.

## Interview script (in order)

The supervisor runs these one at a time, surfacing each as a single-turn Question primitive (SRS §6.6.2). Each answer fills one `expects.required_fields` slot.

### Q1 — `requirements_files`

> "Which requirements doc(s) should I work from? Paste a path, a URL, or drop the file. You can name multiple — comma-separated or one per line."

Acceptance: at least one path that resolves to a UTF-8-decodable file under 5 MB. Multiple paths allowed. Each gets `media_type` auto-detected.

### Q2 — `output_dir`

> "Where should I write the FR markdowns? Default is `./feature-requests/`."

Acceptance: a directory path. Created if missing (BOOT-005). MUST be under the current project root (no parent escapes).

### Q3 — `manifest_path`

> "And the manifest? Default is `<output_dir>/manifest.json`."

Acceptance: a JSON file path. May be inside `output_dir` or alongside.

## Optional-field defaults (no question asked unless user volunteers)

| Field | Default | Override trigger |
| --- | --- | --- |
| `batch_size`     | 3                                        | User says "do all of them" → ask: "I cap at 10 per batch; OK to do 10 then resume?" |
| `caller_persona` | `cuo-cpo`                                | Ignored unless user explicitly invokes a different persona-card. |
| `trace_id`       | auto-generated UUIDv7                     | Use the supplied one if user is replaying a prior trace for debugging. |

## Standalone-mode resume

If `manifest.json` already exists at `manifest_path`, skip the interview and re-enter at the computed phase (PLAN / WORKER / RESUME) per the `Phase computation` table in `SKILL.md`. The interview runs only on genuine first-run starts.

## Exit ramp

If the user replies "actually, I don't have requirements yet — help me write one first," the supervisor MUST NOT proceed with `fr-create`. Instead it routes to a future `cuo/cpo/prd-draft` skill (not yet authored as of v0.2.0) or surfaces "There's no PRD-drafting skill yet; would you like to write the PRD by hand and come back?"

## Audit row

Each Q→A round produces one `genie.action_log` row of `row_kind: question`. The final answer-set produces one `row_kind: act` summarising the inputs the supervisor synthesised before invoking `fr-create`.

## See also

- `HUMAN_SUMMARY.md` — what the user sees AFTER each batch (the other half of standalone-mode UX).
- `references/HITL_PROTOCOL.md` — the broader HITL surface (this interview is the entry-time variant).
