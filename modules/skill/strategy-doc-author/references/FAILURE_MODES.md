# Failure modes — BOOT codes

Version: 1.0.0  Status: Normative for every skill in the SKILL module.

This file is copied verbatim into every skill bundle. Add skill-specific codes only with a leading `<SKILL>-` prefix to avoid collision with the canonical BOOT-NNN set.

---

## §1  Canonical BOOT codes

| Code | Reason | Recovery |
|---|---|---|
| `BOOT-001` | A required input file was not found. | Operator provides the correct path; skill re-runs. |
| `BOOT-002` | An input file was not valid UTF-8 after extraction. | Operator re-encodes the file; skill re-runs. |
| `BOOT-003` | `manifest.json` exists but JSON parse failed. | Operator inspects manifest or restores from `manifest.json.bak`; skill re-runs in PLAN phase. |
| `BOOT-004` | `manifest.json` schema version is not the expected `manifest@N`. | CONTRACT_DRIFT. Operator runs migration or accepts re-PLAN. |
| `BOOT-005` | `output_dir` does not exist and could not be created. | Operator creates the directory or adjusts scope sandbox. |
| `BOOT-006` | The runtime cannot reach a chained skill. | Operator confirms the chain target is installed and reachable; skill re-runs. |
| `BOOT-007` | Mode dispatch ambiguous — author invoked with fields belonging to audit (or vice versa). | Operator splits the invocation into separate skill calls. |
| `BOOT-008` | A required `depends_on_contracts` template is missing or version-mismatched. | Operator updates the contract pin or installs the correct contract version. |

## §2  Drift codes

| Code | Reason | Recovery |
|---|---|---|
| `CONTRACT_DRIFT` | A contract's CONTRACT_ECHO version doesn't match the skill's declared `template_version`. | Operator decides: re-pin or re-run with the new contract. |
| `INPUTS_CHANGED` | `source_hash` differs from the manifest's last-known value. | Skill resets affected artefacts to STALE; operator chooses revert-to-manifest or proceed-with-new-inputs. |
| `STALE_OVERWRITE` | The skill is about to overwrite a PASS or HITL_PAUSE artefact whose source has changed. | HITL escalation; operator confirms or aborts. |
| `EXHAUSTED` | Inner audit loop hit `max_iterations` without converging. | HITL escalation; operator decides whether to ship with warnings or revise the artefact manually. |
| `NO_PROGRESS` | Inner audit loop ran a round with zero auto-fixes and no new `needs_human` issues. | Diagnostic; operator inspects the artefact. |

## §3  Self-audit codes

Emitted by the skill's own self-audit invariants (see `INVARIANTS.md`):

| Code | Reason | Action |
|---|---|---|
| `REFINEMENT_PROPOSAL` | An anomaly signal in `self_audit.anomaly_signals` breached its threshold. | Pipeline pauses; operator reviews per `human_fine_tune` procedure. |

## §4  How a failure mode surfaces

On any BOOT-NNN or drift code:

1. The skill writes a `genie.action_log` row of kind `error` with `code` set to the BOOT identifier and `evidence` containing the offending path / hash / version.
2. The skill emits the code in the response as a single fenced block:
   ```
   FAILURE
   code: BOOT-004
   reason: <human-readable explanation>
   evidence: <path or hash>
   recovery: <suggested next step from §1-§3 above>
   ```
3. The skill exits with `batch_outcome: EXHAUSTED` (for drift codes that cannot proceed) OR `HALTED_HITL` (for codes that need human input).
4. The operator's next invocation re-enters via manifest state — no work is lost.

## §5  Cross-references

- `INVARIANTS.md` (sibling file) — self-audit invariant catalog.
- `references/MANIFEST_SCHEMA.md` (sibling file) — manifest re-entrancy rules that produce most drift codes.
