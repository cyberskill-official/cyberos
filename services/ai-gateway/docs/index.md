---
title: ai - the model gateway · CyberOS
migrated: FR-DOCS-002
---

ai is the platform's single door to language models. No module calls a provider directly: every inference request goes through the gateway, which routes, meters, and audits it.

## Current state

- RouterBackend routes requests across providers; local backends (LM Studio, Ollama) run without any cloud key, so the platform works fully offline. Cloud providers slot in behind the same interface when keys are configured.
- Cost controls: a cost ledger with reconcile, spend caps, and cost-hold expiry - a request that would breach the cap is refused, never silently billed.
- The embed sidecar serves embedding requests for the memory module's brain (deterministic stub in tests; real models in deployment).
- Everything is tenant-scoped and observable; gateway-only model access is a protected invariant no FR may weaken.

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
