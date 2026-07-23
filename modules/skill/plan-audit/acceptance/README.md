# `plan-audit` — acceptance fixtures

Every audit skill ships at least one golden input/output fixture pair so the parity harness can detect regressions. Audit fixtures use STRICT BYTE-EQUALITY because audit output is required to be byte-deterministic (see `REPORT_FORMAT.md` Byte-stability). Add fixtures here as `acceptance/golden-<flow-id>-input.json` and `acceptance/golden-<flow-id>-output.md` (plus the input artefact file itself in `acceptance/<artefact-name>.md`).

## Fixture catalog

| Flow ID | Input artefact | Expected audit | Verdict | What it covers |
|---|---|---|---|---|
| `pass-clean` | `acceptance/pass-clean.md` | `acceptance/pass-clean.audit.md` | `pass` (10/10) | Well-formed greenfield plan@1 passes on first iteration. |
| `one-option-red` | `acceptance/one-option-red.md` | `acceptance/one-option-red.audit.md` | `fail` | `## 3. Options` weighs a single option → `PLAN-OPT-001` (nothing was weighed). |
| `two-decisions-red` | `acceptance/two-decisions-red.md` | `acceptance/two-decisions-red.audit.md` | `fail` | `## 4. Decision` records two decisions → `PLAN-DEC-001`. |
| `empty-out-list-red` | `acceptance/empty-out-list-red.md` | `acceptance/empty-out-list-red.audit.md` | `fail` | `### Out of scope` present but empty → `PLAN-OUT-001`. |
| `no-verdict-needs-human` | `acceptance/no-verdict-needs-human.md` | `acceptance/no-verdict-needs-human.audit.md` | `needs_human` | No recorded operator verdict in `## 4. Decision` → `PLAN-GATE-001`; loop terminates immediately per the AUDIT_LOOP.md override. |
| `brownfield-null-scan-red` | `acceptance/brownfield-null-scan-red.md` | `acceptance/brownfield-null-scan-red.audit.md` | `fail` | `mode: brownfield` with `scan_ref: null` → `PLAN-SAFE-004` (planned against a live repo without scanning it). |
| `unwrapped-quote-red` | `acceptance/unwrapped-quote-red.md` | `acceptance/unwrapped-quote-red.audit.md` | `fail` | Operator text quoted into `## 2. Context` outside `<untrusted_content>` → `PLAN-SAFE-003`. |
| `classless-task-row-red` | `acceptance/classless-task-row-red.md` | `acceptance/classless-task-row-red.audit.md` | `fail` | A `## 6. Proposed Task Set` row without `class: product\|improvement` → `PLAN-SET-002` (create-tasks would have to guess). |

## Running the harness

```bash
cd skill
cargo run -p cyberos-skill-cli -- run plan-audit \
  --input <skill-dir>/acceptance/golden-<flow-id>-input.json \
  --golden-output <skill-dir>/acceptance/golden-<flow-id>-output.md \
  --byte-equality strict
```

The strict byte-equality flag is mandatory for audit fixtures — non-byte-stable output is a determinism breach.

## Adding a fixture

1. Construct or capture the input artefact file under `acceptance/<artefact-name>.md`.
2. Construct the input envelope per `envelopes/input.json`, pointing at the artefact.
3. Run the skill against it manually; capture the resulting `.audit.md` as the golden.
4. Add a row to the catalog above.
5. Bump the skill's `CHANGELOG.md` to note the new fixture.
6. Re-run the harness — it should now pass.

## Anti-patterns

- **Do not** capture a fixture from a non-deterministic run; check `REPORT_FORMAT.md` Byte-stability first.
- **Do not** include real customer data, real person handles, or real money amounts in fixtures.
- **Do not** add a fixture without updating the catalog table above.
- **Do not** rely on `plan-author` being deterministic — capture audit fixtures from synthetic, hand-crafted plan@1 artefacts (the author is judgement-shaped; the audit is byte-deterministic).
- **Do not** hand-edit a fixture to make `PLAN-GATE-001` pass — the verdict line must come from a recorded gate transcript, which is exactly what the rule verifies.
