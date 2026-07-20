        ---
        skill_id: soc2-evidence-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for soc2-evidence-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Audit this soc2 evidence"
- "Check the soc2 evidence for completeness"
- "Verify the soc2 evidence meets the rubric"
- "Re-audit the soc2 evidence"

        ## Negative triggers (MUST NOT route here)

- "Draft a soc2 evidence" → soc2-evidence-author
- "Create the soc2 evidence" → soc2-evidence-author
- "What is the team on-call rotation" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
