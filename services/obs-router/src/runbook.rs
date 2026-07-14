//! Runbook-URL allowlisting (TASK-OBS-007 P2 hardening). The CUO `obs.triage-alert` skill is told to cite
//! only a runbook it actually matched in the corpus and never to invent a URL - but a local model can
//! still echo the example URL from the skill doc (`https://kb/.../rollback-gateway`) or otherwise fabricate
//! one. So CHAT never shows a made-up link, the router keeps a suggested runbook only when its URL is
//! EXACTLY one of the known KB runbook URLs (the allowlist / KB index). An empty allowlist trusts nothing
//! (fail-closed): the runbook is dropped and CHAT shows "Runbook: none", while the alert still routes and
//! pages exactly as before. Exact match (not a host prefix) is deliberate: a fabricated slug on the real
//! KB host must be rejected too.

/// Keep the suggested runbook URL only if it is exactly an allowlisted (known KB) URL; otherwise drop it.
/// Surrounding whitespace is tolerated. An empty allowlist trusts nothing (fail-closed -> `None`).
pub fn sanitize_runbook(candidate: Option<&str>, allowlist: &[String]) -> Option<String> {
    let url = candidate?.trim();
    if url.is_empty() {
        return None;
    }
    if allowlist.iter().any(|allowed| allowed == url) {
        Some(url.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allow() -> Vec<String> {
        vec![
            "https://kb.cyberos.world/runbooks/memory-latency".to_string(),
            "https://kb.cyberos.world/runbooks/rollback-gateway".to_string(),
        ]
    }

    #[test]
    fn keeps_an_exact_allowlisted_url() {
        assert_eq!(
            sanitize_runbook(
                Some("https://kb.cyberos.world/runbooks/memory-latency"),
                &allow()
            ),
            Some("https://kb.cyberos.world/runbooks/memory-latency".to_string())
        );
    }

    #[test]
    fn tolerates_surrounding_whitespace() {
        assert_eq!(
            sanitize_runbook(
                Some("  https://kb.cyberos.world/runbooks/rollback-gateway \n"),
                &allow()
            ),
            Some("https://kb.cyberos.world/runbooks/rollback-gateway".to_string())
        );
    }

    #[test]
    fn drops_the_skill_doc_example_url_a_local_model_copies() {
        // The literal `suggested_runbook.url` example from modules/skill/obs-triage-alert/SKILL.md. The
        // skill says never to invent a URL, but a local model can echo the example - it must be blanked.
        assert_eq!(
            sanitize_runbook(Some("https://kb/.../rollback-gateway"), &allow()),
            None
        );
    }

    #[test]
    fn drops_a_fabricated_slug_on_the_real_host() {
        // Right host, invented page - exact match (not host-prefix) is what catches this.
        assert_eq!(
            sanitize_runbook(
                Some("https://kb.cyberos.world/runbooks/totally-made-up"),
                &allow()
            ),
            None
        );
    }

    #[test]
    fn empty_allowlist_is_fail_closed() {
        assert_eq!(
            sanitize_runbook(
                Some("https://kb.cyberos.world/runbooks/memory-latency"),
                &[]
            ),
            None
        );
    }

    #[test]
    fn none_and_blank_stay_none() {
        assert_eq!(sanitize_runbook(None, &allow()), None);
        assert_eq!(sanitize_runbook(Some("   "), &allow()), None);
    }
}
