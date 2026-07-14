        ---
        skill_id: breach-notification-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for breach-notification-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a breach notification"
- "Create the breach notification"
- "Author a new breach notification"
- "Generate the breach notification"

        ## Negative triggers (MUST NOT route here)

        - "Audit this breach notification" → breach-notification-audit
- "Check the breach notification for completeness" → breach-notification-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
