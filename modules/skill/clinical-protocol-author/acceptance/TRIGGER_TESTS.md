        ---
        skill_id: clinical-protocol-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for clinical-protocol-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a clinical protocol"
- "Create the clinical protocol"
- "Author a new clinical protocol"
- "Generate the clinical protocol"

        ## Negative triggers (MUST NOT route here)

        - "Audit this clinical protocol" → clinical-protocol-audit
- "Check the clinical protocol for completeness" → clinical-protocol-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
