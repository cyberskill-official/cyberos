        ---
        skill_id: repo-context-map-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for repo-context-map-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this repo context map"
- "Check the repo context map for completeness"
- "Verify the repo context map meets the rubric"
- "Re-audit the repo context map"

        ## Negative triggers (MUST NOT route here)

        - "Draft a repo context map" → repo-context-map-author
- "Create the repo context map" → repo-context-map-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
