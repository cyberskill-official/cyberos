        ---
        skill_id: diversity-equity-inclusion-program-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for diversity-equity-inclusion-program-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Audit this diversity equity inclusion program"
- "Check the diversity equity inclusion program for completeness"
- "Verify the diversity equity inclusion program meets the rubric"
- "Re-audit the diversity equity inclusion program"

        ## Negative triggers (MUST NOT route here)

- "Draft a diversity equity inclusion program" → diversity-equity-inclusion-program-author
- "Create the diversity equity inclusion program" → diversity-equity-inclusion-program-author
- "What is the team on-call rotation" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
