        ---
        skill_id: procurement-strategy-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for procurement-strategy-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a procurement strategy"
- "Create the procurement strategy"
- "Author a new procurement strategy"
- "Generate the procurement strategy"

        ## Negative triggers (MUST NOT route here)

- "Audit this procurement strategy" → procurement-strategy-audit
- "Check the procurement strategy for completeness" → procurement-strategy-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
