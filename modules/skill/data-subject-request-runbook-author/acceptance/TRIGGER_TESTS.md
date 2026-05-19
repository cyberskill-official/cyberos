        ---
        skill_id: data-subject-request-runbook-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for data-subject-request-runbook-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a data subject request runbook"
- "Create the data subject request runbook"
- "Author a new data subject request runbook"
- "Generate the data subject request runbook"

        ## Negative triggers (MUST NOT route here)

        - "Audit this data subject request runbook" → data-subject-request-runbook-audit
- "Check the data subject request runbook for completeness" → data-subject-request-runbook-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
