        ---
        skill_id: ai-use-case-portfolio-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for ai-use-case-portfolio-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this ai use case portfolio"
- "Check the ai use case portfolio for completeness"
- "Verify the ai use case portfolio meets the rubric"
- "Re-audit the ai use case portfolio"

        ## Negative triggers (MUST NOT route here)

        - "Draft a ai use case portfolio" → ai-use-case-portfolio-author
- "Create the ai use case portfolio" → ai-use-case-portfolio-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
