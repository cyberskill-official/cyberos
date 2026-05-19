        ---
        skill_id: hire-decision-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for hire-decision-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this hire decision"
- "Check the hire decision for completeness"
- "Verify the hire decision meets the rubric"
- "Re-audit the hire decision"

        ## Negative triggers (MUST NOT route here)

        - "Draft a hire decision" → hire-decision-author
- "Create the hire decision" → hire-decision-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
