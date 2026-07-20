        ---
        skill_id: rhythm-of-business-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for rhythm-of-business-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a rhythm of business"
- "Create the rhythm of business"
- "Author a new rhythm of business"
- "Generate the rhythm of business"

        ## Negative triggers (MUST NOT route here)

- "Audit this rhythm of business" → rhythm-of-business-audit
- "Check the rhythm of business for completeness" → rhythm-of-business-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
