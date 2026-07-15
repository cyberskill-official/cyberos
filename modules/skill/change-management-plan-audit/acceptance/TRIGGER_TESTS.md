        ---
        skill_id: change-management-plan-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for change-management-plan-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this change management plan"
- "Check the change management plan for completeness"
- "Verify the change management plan meets the rubric"
- "Re-audit the change management plan"

        ## Negative triggers (MUST NOT route here)

        - "Draft a change management plan" → change-management-plan-author
- "Create the change management plan" → change-management-plan-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
