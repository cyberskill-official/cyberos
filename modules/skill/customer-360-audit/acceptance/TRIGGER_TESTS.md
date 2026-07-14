        ---
        skill_id: customer-360-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for customer-360-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this customer 360"
- "Check the customer 360 for completeness"
- "Verify the customer 360 meets the rubric"
- "Re-audit the customer 360"

        ## Negative triggers (MUST NOT route here)

        - "Draft a customer 360" → customer-360-author
- "Create the customer 360" → customer-360-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
