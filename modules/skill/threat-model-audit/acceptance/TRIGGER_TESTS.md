        ---
        skill_id: threat-model-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for threat-model-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Audit this threat model"
- "Check the threat model for completeness"
- "Verify the threat model meets the rubric"
- "Re-audit the threat model"

        ## Negative triggers (MUST NOT route here)

- "Draft a threat model" → threat-model-author
- "Create the threat model" → threat-model-author
- "What is the team on-call rotation" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
