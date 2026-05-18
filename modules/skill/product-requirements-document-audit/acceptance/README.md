# `product-requirements-document-audit` — acceptance fixtures

Every audit skill ships at least one golden input/output fixture pair so the parity harness can detect regressions. Audit fixtures use STRICT BYTE-EQUALITY because audit output is required to be byte-deterministic (INV-006). Add fixtures here as `acceptance/golden-<flow-id>-input.json` and `acceptance/golden-<flow-id>-output.md` (plus the input artefact file itself in `acceptance/<artefact-name>.md`).

## Fixture catalog

| Flow ID | Input artefact | Expected audit | Verdict | What it covers |
|---|---|---|---|---|
| `pass-clean` | `acceptance/pass-clean.md` | `acceptance/pass-clean.audit.md` | `pass` | Well-formed artefact passes on first iteration. |
| `auto-fix` | `acceptance/auto-fix-input.md` | `acceptance/auto-fix-input.audit.md` | `pass` | Multiple auto-fixable issues; rubric converges in 2-3 iterations. |
| `hitl-numeric` | `acceptance/hitl-numeric.md` | `acceptance/hitl-numeric.audit.md` | `needs_human` | Unsourced numeric target triggers `QA-NUM-001 → needs_human`. |
| `stale-source` | `acceptance/stale-source.md` (with `provenance.source_hash` mismatched) | `acceptance/stale-source.audit.md` | `needs_human` (`STALE-001`) | Source-hash drift triggers stale handling. |
| `injection-marker` | `acceptance/injection-marker.md` (contains `ignore previous instructions` inside `<untrusted_content>`) | `acceptance/injection-marker.audit.md` | `fail` | `SAFE-003` fires at ≥3 markers → error. |
| `nested-untrusted` | `acceptance/nested-untrusted.md` | `acceptance/nested-untrusted.audit.md` | `fail` | `SAFE-001` fires on nested `<untrusted_content>` blocks. |

## Running the harness

```bash
cd skill
cargo run -p cyberos-skill-cli -- run product-requirements-document-audit \
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

- **Do not** capture a fixture from a non-deterministic run; check `INV-006` first.
- **Do not** include real customer data, real person handles, or real money amounts in fixtures.
- **Do not** add a fixture without updating the catalog table above.
- **Do not** rely on the author skill being deterministic — capture audit fixtures from synthetic, hand-crafted artefacts when the author is non-deterministic.
