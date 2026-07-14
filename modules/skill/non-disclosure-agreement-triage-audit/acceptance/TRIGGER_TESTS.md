        ---
        skill_id: non-disclosure-agreement-triage-audit
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for non-disclosure-agreement-triage-audit

        > Authored via heuristic backfill per TASK-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        - "Audit this non disclosure agreement triage"
- "Check the non disclosure agreement triage for completeness"
- "Verify the non disclosure agreement triage meets the rubric"
- "Re-audit the non disclosure agreement triage"

        ## Negative triggers (MUST NOT route here)

        - "Draft a non disclosure agreement triage" → non-disclosure-agreement-triage-author
- "Create the non disclosure agreement triage" → non-disclosure-agreement-triage-author
- "What is the team on-call rotation" → none

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
