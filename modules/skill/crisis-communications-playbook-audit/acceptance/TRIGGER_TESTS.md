        ---
        skill_id: crisis-communications-playbook-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for crisis-communications-playbook-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this crisis communications playbook"
- "Check the crisis communications playbook for completeness"
- "Verify the crisis communications playbook meets the rubric"
- "Re-audit the crisis communications playbook"

        ## Negative triggers (MUST NOT route here)

        - "Draft a crisis communications playbook" → crisis-communications-playbook-author
- "Create the crisis communications playbook" → crisis-communications-playbook-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
