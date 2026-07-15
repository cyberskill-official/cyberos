        ---
        skill_id: customer-success-engagement-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for customer-success-engagement-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this customer success engagement"
- "Check the customer success engagement for completeness"
- "Verify the customer success engagement meets the rubric"
- "Re-audit the customer success engagement"

        ## Negative triggers (MUST NOT route here)

        - "Draft a customer success engagement" → customer-success-engagement-author
- "Create the customer success engagement" → customer-success-engagement-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
