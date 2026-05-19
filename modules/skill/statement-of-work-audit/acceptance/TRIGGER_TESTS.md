        ---
        skill_id: statement-of-work-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for statement-of-work-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this statement of work"
- "Check the statement of work for completeness"
- "Verify the statement of work meets the rubric"
- "Re-audit the statement of work"

        ## Negative triggers (MUST NOT route here)

        - "Draft a statement of work" → statement-of-work-author
- "Create the statement of work" → statement-of-work-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
