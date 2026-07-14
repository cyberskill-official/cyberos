        ---
        skill_id: bias-audit-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for bias-audit-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a bias audit"
- "Create the bias audit"
- "Author a new bias audit"
- "Generate the bias audit"

        ## Negative triggers (MUST NOT route here)

        - "Audit this bias audit" → bias-audit-audit
- "Check the bias audit for completeness" → bias-audit-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
