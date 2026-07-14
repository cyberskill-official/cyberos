        ---
        skill_id: vietnam-vneid-integration
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for vietnam-vneid-integration

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Reference vietnam vneid integration"
- "Look up vietnam vneid integration"
- "Consult the vietnam vneid integration reference"

        ## Negative triggers (MUST NOT route here)

        - "Run unrelated task" → none
- "What time is it" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
