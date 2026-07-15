        ---
        skill_id: employee-net-promoter-score-program-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for employee-net-promoter-score-program-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a employee net promoter score program"
- "Create the employee net promoter score program"
- "Author a new employee net promoter score program"
- "Generate the employee net promoter score program"

        ## Negative triggers (MUST NOT route here)

        - "Audit this employee net promoter score program" → employee-net-promoter-score-program-audit
- "Check the employee net promoter score program for completeness" → employee-net-promoter-score-program-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
