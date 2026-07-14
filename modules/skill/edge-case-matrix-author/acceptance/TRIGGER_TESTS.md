        ---
        skill_id: edge-case-matrix-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for edge-case-matrix-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a edge case matrix"
- "Create the edge case matrix"
- "Author a new edge case matrix"
- "Generate the edge case matrix"

        ## Negative triggers (MUST NOT route here)

        - "Audit this edge case matrix" → edge-case-matrix-audit
- "Check the edge case matrix for completeness" → edge-case-matrix-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
