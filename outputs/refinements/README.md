# outputs/refinements/

Staging area for §0.4 refinement proposals BEFORE they're promoted to `.cyberos-memory/memories/refinements/REF-NNN-<slug>.md`.

## Workflow

1. **Draft** here as `draft-<slug>.md` — free-form working notes
2. **Decide** with stakeholders (or run `/council` mode per Aspect 3.3 if ambiguous)
3. **Format** to the official REF template: `python3 runtime/tools/cyberos_add.py REF --slug <slug>` reads `.cyberos-memory/meta/templates/REF.md`
4. **Capability + regression eval** scaffolded under `runtime/tests/refinements/REF-NNN/`
5. **Commit** via `brain_writer.py write` — drafts in this dir get garbage-collected by `.gitignore`

## Why staging vs direct write

- Drafts can sit half-finished without polluting the audit ledger
- §0.4 standing rule says "propose in same response" — draft captures the proposal; the formal write happens after eval scaffolding lands
- Drafts excluded from §11.2 deterministic export

## Auto-detected candidates

The Stop-hook at `runtime/hooks/refinement_candidates.py` emits candidates to `.cyberos-memory/memories/drift/<date>-refinement-candidate-*.md`. Review weekly; promote the actionable ones into this dir, then through the workflow above.

## Cleanup

Drafts > 30 days old without promotion: tombstone via `mv draft-<slug>.md .archive-<slug>.md`. The `.archive-` prefix is also gitignored.
