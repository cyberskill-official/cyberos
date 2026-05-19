        ---
        skill_id: product-metrics-review-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for product-metrics-review-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this product metrics review"
- "Check the product metrics review for completeness"
- "Verify the product metrics review meets the rubric"
- "Re-audit the product metrics review"

        ## Negative triggers (MUST NOT route here)

        - "Draft a product metrics review" → product-metrics-review-author
- "Create the product metrics review" → product-metrics-review-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
