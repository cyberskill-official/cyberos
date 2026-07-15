        ---
        skill_id: happiness-program-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for happiness-program-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a happiness program"
- "Create the happiness program"
- "Author a new happiness program"
- "Generate the happiness program"

        ## Negative triggers (MUST NOT route here)

        - "Audit this happiness program" → happiness-program-audit
- "Check the happiness program for completeness" → happiness-program-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
