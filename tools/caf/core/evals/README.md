# core/evals/ — regression gate for AUDIT.md

Every change to `AUDIT.md` must keep this suite green. The harness validates **agent outputs** (a run's `docs/BACKLOG.md` + `docs/HANDOFF.md`) against the machine-checkable subset of the protocol's rules, and proves each rule is **load-bearing** by fault injection: a `B*` fixture plants a known fault set (usually exactly one; `B16` deliberately plants three to verify exact-set reporting), and the validator must catch exactly that set — no more, no less. A trap that stops tripping means a rule has silently died. `G*` fixtures are the precision half: compliant-but-tricky outputs (negated prose, redaction markers, adversarial formatting, minimal-valid) that must NOT trip anything — the suite proves rules fire *and* don't over-fire.

Two checks make the rest load-bearing: `TEMPLATE-NONCONFORMANT` (output that doesn't follow the Phase 2 template can no longer silently escape the other tripwires — BS-12) and the CONFIG preflight (`CONFIG-PLACEHOLDER` / `CONFIG-BAD-ENUM`, which also auto-loads `PROTECTED_AREAS` from the target's AUDIT.md so R3 needs no `--protected` double entry — BS-13).

## Files

| File | Role |
|---|---|
| `code_audit_validator.py` | The validator implementation. Zero dependencies (stdlib only); published to PyPI as `code-audit-validator`. |
| `validate.py` | Repo-side CLI shim over the module above — every documented `python3 core/evals/validate.py …` invocation goes through it. |
| `fixtures/` | `G*` = compliant outputs that must pass. `B*` = fault-injection traps that must fail with declared codes. |
| `rules.json` | Rule registry: rule → AUDIT.md anchor → violation codes → fixtures proving it. Coverage gaps are declared honestly. |
| `baseline.json` | Last recorded matrix (fixture → outcome) pinned to an AUDIT.md version + sha256. |
| `run-evals.sh` | Runner; `--record` refreshes `baseline.json`. |

## Commands

```bash
python3 core/evals/validate.py --all                  # full suite, human output
python3 core/evals/validate.py --all --json           # machine-readable
./core/evals/run-evals.sh --record                    # run + pin baseline to current AUDIT.md
python3 core/evals/validate.py --run <dir>            # validate a real run's docs/ output
python3 core/evals/validate.py --run <dir> --report json   # structured findings export (core/schemas/report.v1.json)
python3 core/evals/validate.py --run <dir> --report sarif  # GitHub code-scanning format
python3 core/evals/validate.py --aggregate r1.json r2.json # portfolio roll-up over report JSONs
python3 core/evals/validate.py --batch targets.yaml        # fleet runner: per-target reports + portfolio.json
python3 core/evals/validate.py --compare prev.json curr.json  # run-over-run regressions (reopened tasks, new codes)
python3 core/evals/validate.py --run <dir> --emit-feedback # feedback@1 calibration record (core/schemas/feedback.v1.json)
python3 core/evals/validate.py --run <dir> --fail-on High  # severity exit-code policy (all violations still printed)
python3 core/evals/scripts/retro-summary.py           # retro scores per protocol version (did each release help?)
python3 core/evals/scripts/retro-summary.py --feedback-dir <field-data>  # + per-version FIELD trend
```

Field-run accuracy evaluation (tiers, metrics, calibration pipeline): [`TESTING-PROTOCOL.md`](./TESTING-PROTOCOL.md).

Point `--run` at the target repo root (or its `docs/`): if the target's `AUDIT.md` is found, its CONFIG is preflighted and `PROTECTED_AREAS` is loaded automatically; `--protected` extends it.

**Runner vs copy mode.** A target needs either an `audit-profile.yaml` with a `config:` section (runner mode — the protocol stays here, launch via `./core/evals/run-audit.sh <target> [agent cmd]`) or a full `AUDIT.md` copy (copy mode — pinned + self-contained; wins precedence when both exist). The preflight, placeholder/enum checks and PROTECTED_AREAS auto-load behave identically for both sources (B17/B18/G08 vs B26/G11).

**Waivers.** A target repo may carry `docs/AUDIT-WAIVERS.yaml` — audit-trailed, *expiring* suppressions (`code` + optional `file`/`match` + `reason` + `approved_by` + mandatory ISO `expires`). A valid waiver suppresses the matched violation and is reported separately; an expired or undated one un-suppresses it AND flags the stale waiver (`WAIVER-EXPIRED`). This is the sanctioned exception channel — eval fixtures, by contrast, may never be weakened.

**Parsing notes (precision boundaries, pinned by fixtures).** Tables inside
``` fences are raw evidence, never artifacts (G07/B19). Tables must use
leading-pipe GFM rows — the exact Phase 2 template shape; pipeless variants
read as nonconformant. Protected-area matching is case-insensitive substring —
keep CONFIG entries specific (`src/billing/`, not `src/`). Artifacts must be
UTF-8 and ≤ 10 MB (`MALFORMED-FILE` otherwise, never a crash).

**Version pinning.** The validator checks the *current* protocol's template.
Validating artifacts produced under an older protocol? Pin the matching tag
(validator and protocol release in lockstep: `v1.2.0` ↔ protocol v1.2.0).

## Adding a fixture

1. Create `core/evals/fixtures/<Gnn|Bnn>-<slug>/` with `fixture.yaml` + `docs/BACKLOG.md` (and `docs/HANDOFF.md` if relevant).
2. `fixture.yaml` is flat `key: value` (no YAML library needed):

   ```yaml
id: B11-my-trap description: one line expect: fail                  # or pass expected_violations: [R1-NO-OUTPUT] exercises_rules: [R1] protected_areas: []
   ```

3. For `expect: fail`, the validator must report **exactly** `expected_violations` — plant one fault per fixture unless the fixture's purpose is exact-set verification (B16). Keep `docs/BACKLOG.md` template-conformant (Mode line + tables or the R7 line) so the planted fault is the only signal; ship a near-miss `G*` sibling when adding a new rule, so precision is pinned alongside recall.
4. Register the fixture in `rules.json` under the rule(s) it exercises — `validate.py --all` fails on registry drift in either direction (BS-10).
5. Run `./core/evals/run-evals.sh --record`.

## What the validator cannot see (declared gaps)

The authoritative register is [`core/improve/BLINDSPOTS.md`](../improve/BLINDSPOTS.md)
— one row per blind spot, with status and evidence. Headlines:

- Whether code changes were genuinely valuable (retro item 9 — human judgment).
- Whether findings were padded (retro item 3 — judgment; the validator only guarantees padding is never *required*).
- Live-agent properties: actual command execution, 3-strike counting, resume behavior (R4). For hard guarantees on those, use deterministic hooks/CI in the target repo, not prompt text.
- Run completeness: `--run` accepts a docs/ directory without HANDOFF.md (legitimate mid-flight). When reviewing a run that claims to be finished, confirm HANDOFF.md exists yourself (BS-09).
