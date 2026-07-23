# `plan-author` — acceptance fixtures

Every author skill ships at least one golden input/output fixture pair so the parity harness can detect regressions. Add fixtures here as `acceptance/golden-<flow-id>-input.json` and `acceptance/golden-<flow-id>-output.md`.

## Fixture catalog

| Flow ID | Input | Expected output | What it covers |
|---|---|---|---|
| `greenfield-happy` | `golden-greenfield-happy-input.json` | `golden-greenfield-happy-output.md` | Idea only, no repo: mode `greenfield`, scan skipped, two options with URL evidence, gate `APPROVE`, plan emitted with `scan_ref: null`. |
| `brownfield-scan-first` | `golden-brownfield-scan-first-input.json` | `golden-brownfield-scan-first-output.md` | Idea + `repo_root` with commits: repo-wide scan runs BEFORE the interview; `scan_ref` resolves; options cite repo paths. |
| `ambiguous-halt` | `golden-ambiguous-halt-input.json` | `golden-ambiguous-halt-output.md` | No `.cyberos/`, no git HEAD, uncommitted source present: `MODE_HALT`, no interview, no artefact (PLAN-AUTHOR-001). |
| `gate-abort` | `golden-gate-abort-input.json` | `golden-gate-abort-output.md` | Operator answers `ABORT` at the §7 gate: `gate_outcome: ABORTED`, zero file ops, `plan_path: null`. |
| `gate-revise` | `golden-gate-revise-input.json` | `golden-gate-revise-output.md` | Operator answers `REVISE: <edits>`: interview loops with the edits, second gate pass `APPROVE`, `gate_outcome: REVISED_THEN_APPROVED`. |
| `idea-already-exists` | `golden-idea-already-exists-input.json` | `golden-idea-already-exists-output.md` | Brownfield scan surfaces the idea already shipped: option set includes a "this exists" option rather than proposing a duplicate. |

## Running the harness

```bash
cd skill
cargo run -p cyberos-skill-cli -- run plan-author \
  --input <skill-dir>/acceptance/golden-<flow-id>-input.json \
  --golden-output <skill-dir>/acceptance/golden-<flow-id>-output.md
```

The harness compares structurally (mode, gate_outcome, section presence, options count, proposed-task-set row shape) rather than byte-for-byte because the author skill emits judgement-shaped output that is not deterministic at the byte level. The matching audit skill IS byte-deterministic and uses strict byte-equality comparison.

## Adding a fixture

1. Construct the input envelope per `envelopes/input.json`.
2. Run the skill against it manually; capture the output envelope + the artefact files into a single `golden-<flow-id>-output.md` containing all relevant outputs.
3. Add a row to the catalog above.
4. Bump the skill's `CHANGELOG.md` to note the new fixture.
5. Re-run the harness — it should now pass.

## Anti-patterns

- **Do not** capture a fixture from a non-PASS run unless the fixture's purpose is to verify failure handling (`ambiguous-halt` and `gate-abort` above are exactly that).
- **Do not** include real customer data, real person handles, or real money amounts in fixtures. Use deterministic synthetic data.
- **Do not** capture a fixture whose gate verdict was not actually recorded — a plan emitted without a verdict is the PLAN-GATE-001 red the audit exists to catch.
- **Do not** add a fixture without updating the catalog table above.
