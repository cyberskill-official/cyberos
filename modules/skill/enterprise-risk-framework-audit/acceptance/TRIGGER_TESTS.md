        ---
        skill_id: enterprise-risk-framework-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for enterprise-risk-framework-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this enterprise risk framework"
- "Check the enterprise risk framework for completeness"
- "Verify the enterprise risk framework meets the rubric"
- "Re-audit the enterprise risk framework"

        ## Negative triggers (MUST NOT route here)

        - "Draft a enterprise risk framework" → enterprise-risk-framework-author
- "Create the enterprise risk framework" → enterprise-risk-framework-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
