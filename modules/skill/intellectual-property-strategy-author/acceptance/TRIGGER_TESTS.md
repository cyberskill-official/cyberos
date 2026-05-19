        ---
        skill_id: intellectual-property-strategy-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for intellectual-property-strategy-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a intellectual property strategy"
- "Create the intellectual property strategy"
- "Author a new intellectual property strategy"
- "Generate the intellectual property strategy"

        ## Negative triggers (MUST NOT route here)

        - "Audit this intellectual property strategy" → intellectual-property-strategy-audit
- "Check the intellectual property strategy for completeness" → intellectual-property-strategy-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
