        ---
        skill_id: mock-contract-test-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for mock-contract-test-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a mock contract test"
- "Create the mock contract test"
- "Author a new mock contract test"
- "Generate the mock contract test"

        ## Negative triggers (MUST NOT route here)

- "Audit this mock contract test" → mock-contract-test-audit
- "Check the mock contract test for completeness" → mock-contract-test-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
