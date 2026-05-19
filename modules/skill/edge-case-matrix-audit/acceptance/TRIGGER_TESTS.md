        ---
        skill_id: edge-case-matrix-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for edge-case-matrix-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this edge case matrix"
- "Check the edge case matrix for completeness"
- "Verify the edge case matrix meets the rubric"
- "Re-audit the edge case matrix"

        ## Negative triggers (MUST NOT route here)

        - "Draft a edge case matrix" → edge-case-matrix-author
- "Create the edge case matrix" → edge-case-matrix-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
