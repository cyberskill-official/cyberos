        ---
        skill_id: analyst-briefing-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for analyst-briefing-author

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Draft a analyst briefing"
- "Create the analyst briefing"
- "Author a new analyst briefing"
- "Generate the analyst briefing"

        ## Negative triggers (MUST NOT route here)

        - "Audit this analyst briefing" → analyst-briefing-audit
- "Check the analyst briefing for completeness" → analyst-briefing-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
