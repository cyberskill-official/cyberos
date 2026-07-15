        ---
        skill_id: penetration-test-report-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for penetration-test-report-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this penetration test report"
- "Check the penetration test report for completeness"
- "Verify the penetration test report meets the rubric"
- "Re-audit the penetration test report"

        ## Negative triggers (MUST NOT route here)

        - "Draft a penetration test report" → penetration-test-report-author
- "Create the penetration test report" → penetration-test-report-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
