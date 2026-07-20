        ---
        skill_id: soc2-evidence-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for soc2-evidence-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a soc2 evidence"
- "Create the soc2 evidence"
- "Author a new soc2 evidence"
- "Generate the soc2 evidence"

        ## Negative triggers (MUST NOT route here)

- "Audit this soc2 evidence" → soc2-evidence-audit
- "Check the soc2 evidence for completeness" → soc2-evidence-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
