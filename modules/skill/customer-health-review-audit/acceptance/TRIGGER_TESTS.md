        ---
        skill_id: customer-health-review-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for customer-health-review-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this customer health review"
- "Check the customer health review for completeness"
- "Verify the customer health review meets the rubric"
- "Re-audit the customer health review"

        ## Negative triggers (MUST NOT route here)

        - "Draft a customer health review" → customer-health-review-author
- "Create the customer health review" → customer-health-review-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
