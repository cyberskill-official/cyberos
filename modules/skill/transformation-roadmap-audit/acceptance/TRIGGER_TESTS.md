        ---
        skill_id: transformation-roadmap-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for transformation-roadmap-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this transformation roadmap"
- "Check the transformation roadmap for completeness"
- "Verify the transformation roadmap meets the rubric"
- "Re-audit the transformation roadmap"

        ## Negative triggers (MUST NOT route here)

        - "Draft a transformation roadmap" → transformation-roadmap-author
- "Create the transformation roadmap" → transformation-roadmap-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
