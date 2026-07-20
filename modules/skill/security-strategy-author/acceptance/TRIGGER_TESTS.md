        ---
        skill_id: security-strategy-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for security-strategy-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a security strategy"
- "Create the security strategy"
- "Author a new security strategy"
- "Generate the security strategy"

        ## Negative triggers (MUST NOT route here)

- "Audit this security strategy" → security-strategy-audit
- "Check the security strategy for completeness" → security-strategy-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
