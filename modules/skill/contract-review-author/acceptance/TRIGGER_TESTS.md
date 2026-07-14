        ---
        skill_id: contract-review-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for contract-review-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a contract review"
- "Create the contract review"
- "Author a new contract review"
- "Generate the contract review"

        ## Negative triggers (MUST NOT route here)

        - "Audit this contract review" → contract-review-audit
- "Check the contract review for completeness" → contract-review-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
