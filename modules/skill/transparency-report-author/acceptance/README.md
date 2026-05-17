# `transparency-report-author` — acceptance fixtures

Every author skill ships at least one golden input/output fixture pair so the parity harness can detect regressions. Add fixtures here as `acceptance/golden-<flow-id>-input.json` and `acceptance/golden-<flow-id>-output.md`.

## Fixture catalog

| Flow ID | Input | Expected output | What it covers |
|---|---|---|---|
| `happy-path-single` | `golden-happy-path-single-input.json` | `golden-happy-path-single-output.md` | Single source file, single artefact, PASS on first iteration. |
| `multi-artefact-batch` | `golden-multi-artefact-batch-input.json` | `golden-multi-artefact-batch-output.md` | Three artefacts in one batch, all PASS. |
| `hitl-pause` | `golden-hitl-pause-input.json` | `golden-hitl-pause-output.md` | One artefact has an unsourced metric; expected HITL_BATCH_REQUEST. |
| `resume` | `golden-resume-input.json` | `golden-resume-output.md` | HITL pause resolved by operator reply; artefact completes on second invocation. |
| `inputs-changed` | `golden-inputs-changed-input.json` | `golden-inputs-changed-output.md` | Source hash drifts mid-batch; STALE handling fires. |

## Running the harness

```bash
cd skill
cargo run -p cyberos-skill-cli -- run transparency-report-author \
  --input <skill-dir>/acceptance/golden-<flow-id>-input.json \
  --golden-output <skill-dir>/acceptance/golden-<flow-id>-output.md
```

The harness compares structurally (artefact IDs, status counts, manifest shape) rather than byte-for-byte because the author skill emits judgement-shaped output that is not deterministic at the byte level. The matching audit skill IS byte-deterministic and uses strict byte-equality comparison.

## Adding a fixture

1. Construct the input envelope per `envelopes/input.json`.
2. Run the skill against it manually; capture the output envelope + the artefact files into a single `golden-<flow-id>-output.md` containing all relevant outputs.
3. Add a row to the catalog above.
4. Bump the skill's `CHANGELOG.md` to note the new fixture.
5. Re-run the harness — it should now pass.

## Anti-patterns

- **Do not** capture a fixture from a non-PASS run unless the fixture's purpose is to verify failure handling.
- **Do not** include real customer data, real person handles, or real money amounts in fixtures. Use deterministic synthetic data.
- **Do not** add a fixture without updating the catalog table above.
