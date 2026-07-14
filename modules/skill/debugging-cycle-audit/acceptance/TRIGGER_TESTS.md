        ---
        skill_id: debugging-cycle-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for debugging-cycle-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this debugging cycle"
- "Check the debugging cycle for completeness"
- "Verify the debugging cycle meets the rubric"
- "Re-audit the debugging cycle"

        ## Negative triggers (MUST NOT route here)

        - "Draft a debugging cycle" → debugging-cycle-author
- "Create the debugging cycle" → debugging-cycle-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
