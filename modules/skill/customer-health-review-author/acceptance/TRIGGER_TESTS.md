        ---
        skill_id: customer-health-review-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for customer-health-review-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a customer health review"
- "Create the customer health review"
- "Author a new customer health review"
- "Generate the customer health review"

        ## Negative triggers (MUST NOT route here)

        - "Audit this customer health review" → customer-health-review-audit
- "Check the customer health review for completeness" → customer-health-review-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
