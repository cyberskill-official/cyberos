        ---
        skill_id: internal-newsletter-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for internal-newsletter-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a internal newsletter"
- "Create the internal newsletter"
- "Author a new internal newsletter"
- "Generate the internal newsletter"

        ## Negative triggers (MUST NOT route here)

        - "Audit this internal newsletter" → internal-newsletter-audit
- "Check the internal newsletter for completeness" → internal-newsletter-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
