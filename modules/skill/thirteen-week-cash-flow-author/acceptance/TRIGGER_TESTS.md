        ---
        skill_id: thirteen-week-cash-flow-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for thirteen-week-cash-flow-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a thirteen week cash flow"
- "Create the thirteen week cash flow"
- "Author a new thirteen week cash flow"
- "Generate the thirteen week cash flow"

        ## Negative triggers (MUST NOT route here)

        - "Audit this thirteen week cash flow" → thirteen-week-cash-flow-audit
- "Check the thirteen week cash flow for completeness" → thirteen-week-cash-flow-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
