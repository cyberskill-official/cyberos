        ---
        skill_id: limited-partner-letter-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for limited-partner-letter-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this limited partner letter"
- "Check the limited partner letter for completeness"
- "Verify the limited partner letter meets the rubric"
- "Re-audit the limited partner letter"

        ## Negative triggers (MUST NOT route here)

        - "Draft a limited partner letter" → limited-partner-letter-author
- "Create the limited partner letter" → limited-partner-letter-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
