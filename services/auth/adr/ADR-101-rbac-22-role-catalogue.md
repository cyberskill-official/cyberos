# ADR-101: 22-role Closed RBAC Catalogue

## Status
Accepted

## Context
The authentication system currently utilizes a stub catalogue of 5 roles. We need to expand this to support the full organizational structure and compliance requirements of our clients, specifically for the 22-role catalogue defined in FR-AUTH-101.

## Decision
We will implement a closed 22-role catalogue in the RBAC system (`Role` enum).
1. The catalogue is closed and compiled into the binary.
2. Any new roles must be justified by a new ADR and added to the source code explicitly.
3. Stub roles remain as a strict prefix of the catalogue for backward compatibility.
4. Reserved roles are protected and cannot be self-assigned.

## Consequences
- **Positive:** Strict control over role definitions. Reduces the attack surface of typo-squatting or ad-hoc roles.
- **Negative:** Requires an application release to introduce new roles.

