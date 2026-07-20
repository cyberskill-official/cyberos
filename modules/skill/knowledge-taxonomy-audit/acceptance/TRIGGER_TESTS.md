        ---
        skill_id: knowledge-taxonomy-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for knowledge-taxonomy-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Audit this knowledge taxonomy"
- "Check the knowledge taxonomy for completeness"
- "Verify the knowledge taxonomy meets the rubric"
- "Re-audit the knowledge taxonomy"

        ## Negative triggers (MUST NOT route here)

- "Draft a knowledge taxonomy" → knowledge-taxonomy-author
- "Create the knowledge taxonomy" → knowledge-taxonomy-author
- "What is the team on-call rotation" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
