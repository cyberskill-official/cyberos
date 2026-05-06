# `cyberos/runtime/` — Runtime build plan + future code

> **Currently:** a build plan for engineering hand-off. Not running code.
>
> **Eventually:** the executable runtime that turns the registry's scaffolds into a live system.

## Read order

1. **[`PLAN.md`](./PLAN.md)** — what the runtime does, what phases land it, what's the critical path. Start here.
2. **[`INTERFACES.md`](./INTERFACES.md)** — public surfaces every skill sees. The shim's contract.
3. **[`BUILD_ORDER.md`](./BUILD_ORDER.md)** — concrete sequence with definition-of-done per phase.

## What this folder is NOT

- Not the registry. The registry (`cyberos/docs/skills/` + `cyberos/docs/contracts/`) is the spec; this folder is the build plan + eventually the implementation.
- Not the architectural decision record. PRD + SRS .docx files in `cyberos/docs/` are the architectural source of truth.
- Not a place to redesign the system. If the design needs to change, change the registry first; implementation follows.

## Contributing

When you start writing code under this folder:

1. Pick a phase from `BUILD_ORDER.md` that's currently unstarted.
2. Read its definition-of-done.
3. Write code under `cyberos/runtime/python/<component>/` or `cyberos/runtime/node/<component>/`.
4. Don't modify the registry while implementing. If you find the spec is wrong, file a registry CHANGELOG entry first.
5. Capture lessons learned as `memories/refinements/REF-NNN-*.md` entries in BRAIN.

## Status

| Phase | Status |
| --- | --- |
| A — CCSM canonicalisation | ✅ done (registry v0.2.0) |
| B — Transpilers | 🔵 planned |
| C — Host shim library | 🔵 planned |
| D — Equivalence test matrix | 🔵 planned |
| E — Partner connector pipeline | 🔵 planned (gated) |
| F — LangGraph supervisor | 🔵 planned |
| G — `genie.action_log` | 🔵 planned |
| H — NATS event bus | 🔵 planned |
| I — Auto-refinement engine | 🔵 planned |
| J — Acceptance-test harness | 🔵 planned |
| K — BRAIN MCP server | 🔵 planned |
| L — KB MCP server | 🔵 planned |
| M — PROJ MCP server | 🔵 planned |
| N — CHAT MCP server | 🔵 planned |
| O — EMAIL MCP server | 🔵 planned |

When phase J turns ✅, this folder retires and gets archived.
