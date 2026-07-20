        ---
        skill_id: privacy-impact-assessment-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for privacy-impact-assessment-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a privacy impact assessment"
- "Create the privacy impact assessment"
- "Author a new privacy impact assessment"
- "Generate the privacy impact assessment"

        ## Negative triggers (MUST NOT route here)

- "Audit this privacy impact assessment" → privacy-impact-assessment-audit
- "Check the privacy impact assessment for completeness" → privacy-impact-assessment-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
