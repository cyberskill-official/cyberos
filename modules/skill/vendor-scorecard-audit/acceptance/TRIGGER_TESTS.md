        ---
        skill_id: vendor-scorecard-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for vendor-scorecard-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this vendor scorecard"
- "Check the vendor scorecard for completeness"
- "Verify the vendor scorecard meets the rubric"
- "Re-audit the vendor scorecard"

        ## Negative triggers (MUST NOT route here)

        - "Draft a vendor scorecard" → vendor-scorecard-author
- "Create the vendor scorecard" → vendor-scorecard-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
