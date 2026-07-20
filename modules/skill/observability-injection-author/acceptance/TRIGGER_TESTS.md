        ---
        skill_id: observability-injection-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for observability-injection-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a observability injection"
- "Create the observability injection"
- "Author a new observability injection"
- "Generate the observability injection"

        ## Negative triggers (MUST NOT route here)

- "Audit this observability injection" → observability-injection-audit
- "Check the observability injection for completeness" → observability-injection-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
