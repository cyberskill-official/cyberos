        ---
        skill_id: requirements-traceability-matrix-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for requirements-traceability-matrix-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a requirements traceability matrix"
- "Create the requirements traceability matrix"
- "Author a new requirements traceability matrix"
- "Generate the requirements traceability matrix"

        ## Negative triggers (MUST NOT route here)

        - "Audit this requirements traceability matrix" → requirements-traceability-matrix-audit
- "Check the requirements traceability matrix for completeness" → requirements-traceability-matrix-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
