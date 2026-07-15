        ---
        skill_id: net-promoter-score-program-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for net-promoter-score-program-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this net promoter score program"
- "Check the net promoter score program for completeness"
- "Verify the net promoter score program meets the rubric"
- "Re-audit the net promoter score program"

        ## Negative triggers (MUST NOT route here)

        - "Draft a net promoter score program" → net-promoter-score-program-author
- "Create the net promoter score program" → net-promoter-score-program-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
