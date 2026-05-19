        ---
        skill_id: pipeline-report-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for pipeline-report-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a pipeline report"
- "Create the pipeline report"
- "Author a new pipeline report"
- "Generate the pipeline report"

        ## Negative triggers (MUST NOT route here)

        - "Audit this pipeline report" → pipeline-report-audit
- "Check the pipeline report for completeness" → pipeline-report-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
