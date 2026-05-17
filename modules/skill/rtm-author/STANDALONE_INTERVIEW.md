# `rtm-author` — standalone interview

When the skill is invoked without a fully-formed envelope (e.g. directly in chat), it runs a short interview to fill the required fields.

## Required fields

| Field | Question | Acceptable answer |
|---|---|---|
| `source_files` | "Which source file(s) should I read to draft the `RTM`?" | One or more existing paths or URLs. The skill confirms each path resolves and is UTF-8 readable. |
| `output_dir` | "Where should I write the `RTM` markdown(s)?" | An absolute or project-relative directory path. The skill creates the directory if it does not exist (subject to scope sandbox). |

## Optional fields

| Field | Question | Default |
|---|---|---|
| `manifest_path` | "Where should I write the manifest? (default: `<output_dir>/manifest.json`)" | `<output_dir>/manifest.json` |
| `batch_size` | "How many `RTM`s per batch? (default 3, max 10)" | 3 |
| `caller_persona` | "Who is asking? (cuo-cpo / cuo-cto / cuo-clo / cuo-cseco / cuo-coo / cuo-ceo)" | `cuo-cpo` |
| `chain_to` | "Should I chain to the audit skill afterwards? (default yes)" | `['rtm-audit']` |

## Interview flow

1. Ask the required questions in order. Pause after each.
2. Validate each answer (existence checks, type checks) before moving on.
3. Echo the assembled envelope back to the user as a single fenced block titled `ENVELOPE_PROPOSED`.
4. Ask: "Begin? (yes / revise X / abort)".
5. On `yes`, emit `CONTRACT_ECHO` and start PLAN phase.
6. On `revise X`, re-ask only the X field.
7. On `abort`, exit cleanly with `INTERVIEW_ABORTED` and no file ops.

## Anti-patterns

- **Do not** ask all questions in one mega-message; users get overwhelmed.
- **Do not** auto-fill optional fields silently; show them in `ENVELOPE_PROPOSED`.
- **Do not** skip the `ENVELOPE_PROPOSED` confirmation — operators frequently catch typos here.
