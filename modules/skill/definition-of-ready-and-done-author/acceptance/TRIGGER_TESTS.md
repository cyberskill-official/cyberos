        ---
        skill_id: definition-of-ready-and-done-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for definition-of-ready-and-done-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a definition of ready and done"
- "Create the definition of ready and done"
- "Author a new definition of ready and done"
- "Generate the definition of ready and done"

        ## Negative triggers (MUST NOT route here)

        - "Audit this definition of ready and done" → definition-of-ready-and-done-audit
- "Check the definition of ready and done for completeness" → definition-of-ready-and-done-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
