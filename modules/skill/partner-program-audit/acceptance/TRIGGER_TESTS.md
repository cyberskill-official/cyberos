        ---
        skill_id: partner-program-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for partner-program-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this partner program"
- "Check the partner program for completeness"
- "Verify the partner program meets the rubric"
- "Re-audit the partner program"

        ## Negative triggers (MUST NOT route here)

        - "Draft a partner program" → partner-program-author
- "Create the partner program" → partner-program-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
