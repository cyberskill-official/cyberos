        ---
        skill_id: definition-of-ready-and-done-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for definition-of-ready-and-done-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this definition of ready and done"
- "Check the definition of ready and done for completeness"
- "Verify the definition of ready and done meets the rubric"
- "Re-audit the definition of ready and done"

        ## Negative triggers (MUST NOT route here)

        - "Draft a definition of ready and done" → definition-of-ready-and-done-author
- "Create the definition of ready and done" → definition-of-ready-and-done-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
