        ---
        skill_id: go-to-market-plan-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for go-to-market-plan-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a go to market plan"
- "Create the go to market plan"
- "Author a new go to market plan"
- "Generate the go to market plan"

        ## Negative triggers (MUST NOT route here)

        - "Audit this go to market plan" → go-to-market-plan-audit
- "Check the go to market plan for completeness" → go-to-market-plan-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
