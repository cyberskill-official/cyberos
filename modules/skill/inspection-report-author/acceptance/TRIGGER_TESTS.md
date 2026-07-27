# TRIGGER_TESTS: inspection-report-author

## Must trigger

- "inspect this repository"
- "run a full audit of this codebase"
- "review this project end to end and tell me what is wrong"
- "what would a senior engineer flag in this repo"
- "give me an improvement backlog for this project"
- "check this repo across every engineering discipline"
- "/inspect"

## Must not trigger

- "audit this inspection report" -> inspection-report-audit
- "check the inspection against the rubric" -> inspection-report-audit
- "fix the issues you found" -> /harden
- "review this pull request" -> code-review-author
- "threat model this service" -> threat-model-author
- "write a test strategy" -> test-strategy-author
- "why is this function slow" -> ordinary debugging, no skill

## Boundary cases

"audit this repo" triggers this skill; "audit this report" does not. The
distinguishing noun is the object of the audit, not the verb.

"inspect and fix" triggers this skill only, then halts and names /harden. The
skill never remediates even when the request asks for both in one sentence.

"quick look at this repo" triggers this skill. There is no shallow mode: the
75-row ledger (spec ≥1.2) is the contract. If the user wants less, say what the
full pass costs and let them decide.
