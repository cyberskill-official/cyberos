        ---
        skill_id: data-strategy-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for data-strategy-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this data strategy"
- "Check the data strategy for completeness"
- "Verify the data strategy meets the rubric"
- "Re-audit the data strategy"

        ## Negative triggers (MUST NOT route here)

        - "Draft a data strategy" → data-strategy-author
- "Create the data strategy" → data-strategy-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
