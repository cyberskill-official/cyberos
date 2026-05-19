        ---
        skill_id: objectives-and-key-results-set-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for objectives-and-key-results-set-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a objectives and key results set"
- "Create the objectives and key results set"
- "Author a new objectives and key results set"
- "Generate the objectives and key results set"

        ## Negative triggers (MUST NOT route here)

        - "Audit this objectives and key results set" → objectives-and-key-results-set-audit
- "Check the objectives and key results set for completeness" → objectives-and-key-results-set-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
