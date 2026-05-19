        ---
        skill_id: churn-analysis-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for churn-analysis-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a churn analysis"
- "Create the churn analysis"
- "Author a new churn analysis"
- "Generate the churn analysis"

        ## Negative triggers (MUST NOT route here)

        - "Audit this churn analysis" → churn-analysis-audit
- "Check the churn analysis for completeness" → churn-analysis-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
