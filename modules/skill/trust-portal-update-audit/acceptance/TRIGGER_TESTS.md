        ---
        skill_id: trust-portal-update-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for trust-portal-update-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Audit this trust portal update"
- "Check the trust portal update for completeness"
- "Verify the trust portal update meets the rubric"
- "Re-audit the trust portal update"

        ## Negative triggers (MUST NOT route here)

- "Draft a trust portal update" → trust-portal-update-author
- "Create the trust portal update" → trust-portal-update-author
- "What is the team on-call rotation" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
