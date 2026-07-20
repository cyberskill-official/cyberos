        ---
        skill_id: campaign-plan-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for campaign-plan-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Audit this campaign plan"
- "Check the campaign plan for completeness"
- "Verify the campaign plan meets the rubric"
- "Re-audit the campaign plan"

        ## Negative triggers (MUST NOT route here)

- "Draft a campaign plan" → campaign-plan-author
- "Create the campaign plan" → campaign-plan-author
- "What is the team on-call rotation" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
