        ---
        skill_id: product-requirements-document-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for product-requirements-document-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this product requirements document"
- "Check the product requirements document for completeness"
- "Verify the product requirements document meets the rubric"
- "Re-audit the product requirements document"

        ## Negative triggers (MUST NOT route here)

        - "Draft a product requirements document" → product-requirements-document-author
- "Create the product requirements document" → product-requirements-document-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
