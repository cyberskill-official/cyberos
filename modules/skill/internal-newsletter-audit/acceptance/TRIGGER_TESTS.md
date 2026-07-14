        ---
        skill_id: internal-newsletter-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for internal-newsletter-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this internal newsletter"
- "Check the internal newsletter for completeness"
- "Verify the internal newsletter meets the rubric"
- "Re-audit the internal newsletter"

        ## Negative triggers (MUST NOT route here)

        - "Draft a internal newsletter" → internal-newsletter-author
- "Create the internal newsletter" → internal-newsletter-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
