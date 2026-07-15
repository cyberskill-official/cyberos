        ---
        skill_id: board-deck-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for board-deck-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this board deck"
- "Check the board deck for completeness"
- "Verify the board deck meets the rubric"
- "Re-audit the board deck"

        ## Negative triggers (MUST NOT route here)

        - "Draft a board deck" → board-deck-author
- "Create the board deck" → board-deck-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
