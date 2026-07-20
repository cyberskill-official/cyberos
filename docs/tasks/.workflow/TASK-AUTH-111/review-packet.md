# TASK-AUTH-111 review packet - status-drift reconciliation (2026-07-12)

Implementation shipped at bc7af7b ("fix(auth): take the SSO display name from the ID token, not the email") by a parallel session; status never flipped. Clause verification of the existing code:

#1/#2 resolution chain (name -> given+family -> preferred_username -> email local-part -> email) in ONE module, display_name::resolve, consumed by oidc.rs:512 PASS; #3 no transformation - module doctrine + a test guarding against future .to_title_case() PASS; #4 no-clobber refresh only when stored value was never-a-name (heal()) PASS; #5 self-healing on next sign-in, no migration PASS (heal on the returning path, warn-not-block oidc.rs:528-530); #6 saml.rs same defect fixed through the SAME resolver (saml.rs:193-202, "ONE resolver decide") PASS; #7 no reconstruction migration PASS (none exists); #8 rung-only debug logging, never the name (tracing::debug!(rung, ...)) PASS - exactly as specified; #9 no picture claim - not a field on Profile, plus test the_picture_claim_has_nowhere_to_land PASS.

Tests: 6 unit tests in display_name.rs incl. the anti-prettify canary and the picture-cannot-land proof. Evidence gap identical to TASK-CHAT-269: no Rust toolchain in this sandbox - operator confirms via CI at bc7af7b or `cargo test -p auth`.
