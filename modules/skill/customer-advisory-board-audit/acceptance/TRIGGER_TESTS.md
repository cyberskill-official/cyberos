        ---
        skill_id: customer-advisory-board-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for customer-advisory-board-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this customer advisory board"
- "Check the customer advisory board for completeness"
- "Verify the customer advisory board meets the rubric"
- "Re-audit the customer advisory board"

        ## Negative triggers (MUST NOT route here)

        - "Draft a customer advisory board" → customer-advisory-board-author
- "Create the customer advisory board" → customer-advisory-board-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
