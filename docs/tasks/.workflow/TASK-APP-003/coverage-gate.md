# TASK-APP-003 — coverage gate (testing phase, 2026-07-13)

- T4: entitlement lint positive — OK 2/2 justified. PASS
- T5: lint negative (unjustified camera key) — fires. PASS
- T6: parses ×4 (workflow YAML, 2 plists XML, overlay JSON). PASS
- T7: Developer ID config untouched (AC #3, 0-diff). PASS
- Floor: run-gates.sh GREEN. Coverage N/A (declared).

TRACE closure: §1 #1✅(T7+overlay) #2✅(12-row audit, lint-enforced) #3✅(inherit plist + helper loop) #4✅(workflow contract, exercised at first MAS_RELEASE run) #5✅(gate + anchor) #6✅(two distinct identity secrets) #7✅(nothing acquired) #8✅(answer sheet; your pending-human rows).
Deferred: AC #4/#5 need macOS + real certs (spec §5 sanctions the documented deferral). **Standing hard blocker #1: updater-exclusion follow-up FR before MAS_RELEASE ever flips** — acceptance of this FR ships the channel inert, not the flag.

**Machine gates green → HALTED at HITL gate 2. Final acceptance verdict is yours.**
