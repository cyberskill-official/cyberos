        ---
        skill_id: release-notes-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for release-notes-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this release notes"
- "Check the release notes for completeness"
- "Verify the release notes meets the rubric"
- "Re-audit the release notes"

        ## Negative triggers (MUST NOT route here)

        - "Draft a release notes" → release-notes-author
- "Create the release notes" → release-notes-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
