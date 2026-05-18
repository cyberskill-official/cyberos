---
template: software-design-document@1
title: <Component or system> — Software Design Description
component_or_system: <Name>
sdd_version: 1.0.0
linked_srs: ./srs.md
linked_adrs: [./adrs/ADR-0001.md]
provenance: { source_path: ./srs.md, source_hash: sha256:<hash> }
created_at: 2026-MM-DDTHH:MM:SS+07:00
author: @<author>
api_versioning_policy: url_path    # url_path | header | content_negotiation | none_applicable
---

# <Component or system> — Software Design Description

## 1. Introduction
Purpose, scope, design overview.

## 2. Context Viewpoint
System boundary, external interfaces.

## 3. Composition Viewpoint
Component decomposition. Each component traces to ≥1 SRS REQ-ID via `traces_to:`.

## 4. Logical Viewpoint
Class / object model, key abstractions.

## 5. Information Viewpoint
Data model, persistence schema.

## 6. Interface Viewpoint
API specs (OpenAPI link), message formats.

## 7. Patterns Viewpoint
Design patterns applied, with rationale.

## 8. Interaction Viewpoint
Sequence diagrams for primary flows.

## 9. State Dynamics Viewpoint
State machines for stateful components.

## 10. Algorithm Viewpoint
Algorithms with Big-O complexity where non-obvious.

## 11. Resource Viewpoint
Memory / CPU / storage / network expectations.

<!-- ## 12. API Specification           — when component exposes HTTP/gRPC/GraphQL -->
<!-- ## 13. Persistence Design          — when component persists data -->
<!-- ## 14. UI Design                   — when component has a UI -->
<!-- ## 15. Performance Design          — when component is perf-critical -->
<!-- ## 16. Backwards-Compatibility Strategy  — when component is public-facing -->
