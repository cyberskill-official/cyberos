# CUO docs

* [`AGENTS.md`](AGENTS.md) — protocol spec (RFC-style, normative)
* [`SPEC.md`](SPEC.md) — formal contract summary; points at AGENTS.md
* [`ROUTING.md`](ROUTING.md) — rule-based heuristics + Phase 2 LLM-driven design
* [`CHANGELOG.md`](CHANGELOG.md) — release history, newest-first

The CUO is the agentic orchestrator: it consumes the memory and skill modules and routes natural-language requests to skills. Every routing decision becomes an audit row in the BRAIN, so the orchestrator's behaviour is replayable and reviewable.
