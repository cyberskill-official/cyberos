# CHAT improvement ledger (append-only)

One entry per task that reaches review. Never edit or delete past entries; corrections get a new entry.
Template:

## T-0NN <title> - <date> - <agent/session>

- Branch/commits: auto/chat-enterprise <sha..sha>
- What changed: 2-4 lines, files touched
- Gate evidence: exact commands run + result (fmt/clippy/test counts, tsc/vite, smoke names + pass counts,
  migration apply output)
- Acceptance evidence: each acceptance check from the task spec, with how it was proven (test name, curl
  output summary, screenshot note)
- Deltas from spec: anything the code forced us to do differently (or "none")
- Left open: follow-ups filed (or "none")

---

(no entries yet)
