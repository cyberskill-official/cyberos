        ---
        skill_id: enterprise-risk-framework-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for enterprise-risk-framework-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a enterprise risk framework"
- "Create the enterprise risk framework"
- "Author a new enterprise risk framework"
- "Generate the enterprise risk framework"

        ## Negative triggers (MUST NOT route here)

        - "Audit this enterprise risk framework" → enterprise-risk-framework-audit
- "Check the enterprise risk framework for completeness" → enterprise-risk-framework-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
