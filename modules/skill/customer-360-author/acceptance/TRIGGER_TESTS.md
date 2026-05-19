        ---
        skill_id: customer-360-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for customer-360-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a customer 360"
- "Create the customer 360"
- "Author a new customer 360"
- "Generate the customer 360"

        ## Negative triggers (MUST NOT route here)

        - "Audit this customer 360" → customer-360-audit
- "Check the customer 360 for completeness" → customer-360-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
