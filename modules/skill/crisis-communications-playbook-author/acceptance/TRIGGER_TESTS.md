        ---
        skill_id: crisis-communications-playbook-author
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for crisis-communications-playbook-author

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

- "Draft a crisis communications playbook"
- "Create the crisis communications playbook"
- "Author a new crisis communications playbook"
- "Generate the crisis communications playbook"

        ## Negative triggers (MUST NOT route here)

- "Audit this crisis communications playbook" → crisis-communications-playbook-audit
- "Check the crisis communications playbook for completeness" → crisis-communications-playbook-audit
- "What is our company holiday schedule" → none

        ## Authoring notes

- Triggers derived from skill name + role (author/audit) via the heuristic backfill script. They are conservative — refine with OBS-observed real user phrasings during the next natural fine-tune cycle.
- Re-author when classifier_version MAJOR-bumps.
