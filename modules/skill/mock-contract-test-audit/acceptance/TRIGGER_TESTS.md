        ---
        skill_id: mock-contract-test-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for mock-contract-test-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this mock contract test"
- "Check the mock contract test for completeness"
- "Verify the mock contract test meets the rubric"
- "Re-audit the mock contract test"

        ## Negative triggers (MUST NOT route here)

        - "Draft a mock contract test" → mock-contract-test-author
- "Create the mock contract test" → mock-contract-test-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
