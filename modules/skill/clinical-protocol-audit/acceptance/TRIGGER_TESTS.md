        ---
        skill_id: clinical-protocol-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for clinical-protocol-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this clinical protocol"
- "Check the clinical protocol for completeness"
- "Verify the clinical protocol meets the rubric"
- "Re-audit the clinical protocol"

        ## Negative triggers (MUST NOT route here)

        - "Draft a clinical protocol" → clinical-protocol-author
- "Create the clinical protocol" → clinical-protocol-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
