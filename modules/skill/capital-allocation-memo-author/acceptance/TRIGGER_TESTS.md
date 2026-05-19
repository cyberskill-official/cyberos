        ---
        skill_id: capital-allocation-memo-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for capital-allocation-memo-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a capital allocation memo"
- "Create the capital allocation memo"
- "Author a new capital allocation memo"
- "Generate the capital allocation memo"

        ## Negative triggers (MUST NOT route here)

        - "Audit this capital allocation memo" → capital-allocation-memo-audit
- "Check the capital allocation memo for completeness" → capital-allocation-memo-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
