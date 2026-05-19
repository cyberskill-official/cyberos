        ---
        skill_id: limited-partner-letter-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for limited-partner-letter-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a limited partner letter"
- "Create the limited partner letter"
- "Author a new limited partner letter"
- "Generate the limited partner letter"

        ## Negative triggers (MUST NOT route here)

        - "Audit this limited partner letter" → limited-partner-letter-audit
- "Check the limited partner letter for completeness" → limited-partner-letter-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
