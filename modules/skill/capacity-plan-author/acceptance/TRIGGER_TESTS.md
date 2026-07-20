        ---
        skill_id: capacity-plan-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for capacity-plan-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a capacity plan"
- "Create the capacity plan"
- "Author a new capacity plan"
- "Generate the capacity plan"

        ## Negative triggers (MUST NOT route here)

- "Audit this capacity plan" → capacity-plan-audit
- "Check the capacity plan for completeness" → capacity-plan-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
