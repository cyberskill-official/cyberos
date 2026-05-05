# `fr-audit` standalone interview script

> Used when `fr-audit` is invoked **standalone** (no upstream pipeline envelope) — to fill `expects.required_fields` from the user via chat. When invoked **chained** (most often after `fr-create`), this file is ignored.

## Mode detection

Same rules as `fr-create/STANDALONE_INTERVIEW.md` §"Mode detection".

## Interview script (in order)

### Q1 — `fr_paths`

> "Which FR(s) should I audit? Paste a path, a file glob, or drop the files. You can name multiple — comma-separated or one per line."

Acceptance: at least one path that resolves to a UTF-8 markdown file whose frontmatter declares `template: feature_request@1`. Glob patterns expand and de-dupe (case-insensitive).

If a path doesn't carry the `template:` literal, surface:
> "I can audit FRs but `<path>` doesn't look like a `feature_request@1` document — its frontmatter doesn't declare `template: feature_request@1`. Skip it, or treat it as one anyway?"

## Optional-field defaults

| Field | Default | Override trigger |
| --- | --- | --- |
| `rubric_version`    | `audit_rubric@2.0` (the version in this folder's `RUBRIC.md`) | User says "use the v1 rubric" → ask: "v1 was retired on 2026-04-22; using it produces an advisory-only report. OK?" |
| `upstream_context`  | `null`                                          | Auto-populated when chained from `fr-create`. |
| `trace_id`          | auto-generated UUIDv7                            | Use the supplied one if user is replaying. |

## Standalone-mode resume

If a `*.audit.md` already exists for any of the requested FRs AND its `audited_file_sha256` matches the FR's current SHA-256, surface:

> "FR-007 was audited 2 hours ago and the file hasn't changed since. Skip, force re-audit anyway, or only audit the others?"

Default to skip. Re-audits are fully-deterministic so there's no risk to forcing — but it costs an LLM call.

## Exit ramp

If the user replies "actually I haven't written the FRs yet — generate them first", the supervisor MUST NOT proceed with `fr-audit`. Instead it routes to `cuo/cpo/fr-create` with the same `output_dir` for chaining back.

## Audit row

Each Q→A round produces one `genie.action_log` row of `row_kind: question`. The synthesised input envelope produces one `row_kind: act` row before invocation.

## See also

- `HUMAN_SUMMARY.md` — chat output after the audit batch completes.
- `RUBRIC.md` — the rule catalogue this skill audits against.
- `references/HITL_PROTOCOL.md` — the HITL pause format.
