        ---
        skill_id: change-management-plan-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for change-management-plan-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a change management plan"
- "Create the change management plan"
- "Author a new change management plan"
- "Generate the change management plan"

        ## Negative triggers (MUST NOT route here)

        - "Audit this change management plan" → change-management-plan-audit
- "Check the change management plan for completeness" → change-management-plan-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
