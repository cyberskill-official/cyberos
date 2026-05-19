        ---
        skill_id: operating-model-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for operating-model-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this operating model"
- "Check the operating model for completeness"
- "Verify the operating model meets the rubric"
- "Re-audit the operating model"

        ## Negative triggers (MUST NOT route here)

        - "Draft a operating model" → operating-model-author
- "Create the operating model" → operating-model-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
