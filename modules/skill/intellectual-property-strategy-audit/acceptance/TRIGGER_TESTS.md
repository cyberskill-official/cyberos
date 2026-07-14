        ---
        skill_id: intellectual-property-strategy-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for intellectual-property-strategy-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this intellectual property strategy"
- "Check the intellectual property strategy for completeness"
- "Verify the intellectual property strategy meets the rubric"
- "Re-audit the intellectual property strategy"

        ## Negative triggers (MUST NOT route here)

        - "Draft a intellectual property strategy" → intellectual-property-strategy-author
- "Create the intellectual property strategy" → intellectual-property-strategy-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
