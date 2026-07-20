        ---
        skill_id: project-plan-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for project-plan-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Audit this project plan"
- "Check the project plan for completeness"
- "Verify the project plan meets the rubric"
- "Re-audit the project plan"

        ## Negative triggers (MUST NOT route here)

- "Draft a project plan" → project-plan-author
- "Create the project plan" → project-plan-author
- "What is the team on-call rotation" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
