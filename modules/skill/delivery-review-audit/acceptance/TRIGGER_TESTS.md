        ---
        skill_id: delivery-review-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for delivery-review-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this delivery review"
- "Check the delivery review for completeness"
- "Verify the delivery review meets the rubric"
- "Re-audit the delivery review"

        ## Negative triggers (MUST NOT route here)

        - "Draft a delivery review" → delivery-review-author
- "Create the delivery review" → delivery-review-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
