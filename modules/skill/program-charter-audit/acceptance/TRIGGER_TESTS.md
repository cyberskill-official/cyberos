        ---
        skill_id: program-charter-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for program-charter-audit

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this program charter"
- "Check the program charter for completeness"
- "Verify the program charter meets the rubric"
- "Re-audit the program charter"

        ## Negative triggers (MUST NOT route here)

        - "Draft a program charter" → program-charter-author
- "Create the program charter" → program-charter-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
