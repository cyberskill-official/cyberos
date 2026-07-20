        ---
        skill_id: software-design-document-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for software-design-document-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a software design document"
- "Create the software design document"
- "Author a new software design document"
- "Generate the software design document"

        ## Negative triggers (MUST NOT route here)

- "Audit this software design document" → software-design-document-audit
- "Check the software design document for completeness" → software-design-document-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
