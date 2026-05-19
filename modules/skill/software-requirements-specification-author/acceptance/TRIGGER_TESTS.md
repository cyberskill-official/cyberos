        ---
        skill_id: software-requirements-specification-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for software-requirements-specification-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a software requirements specification"
- "Create the software requirements specification"
- "Author a new software requirements specification"
- "Generate the software requirements specification"

        ## Negative triggers (MUST NOT route here)

        - "Audit this software requirements specification" → software-requirements-specification-audit
- "Check the software requirements specification for completeness" → software-requirements-specification-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
