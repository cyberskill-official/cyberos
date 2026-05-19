//! FR-EMAIL-001 §1 #8 — RFC 5322 subject normalisation for thread merge.

use cyberos_email::repo::messages::normalise_subject;

#[test]
fn strips_re_prefix_single() {
    assert_eq!(normalise_subject(Some("Re: hello")).as_deref(), Some("hello"));
}

#[test]
fn strips_re_prefix_chain() {
    assert_eq!(normalise_subject(Some("Re: Re: Re: hello")).as_deref(), Some("hello"));
}

#[test]
fn strips_fwd_prefix() {
    assert_eq!(normalise_subject(Some("Fwd: hello")).as_deref(), Some("hello"));
    assert_eq!(normalise_subject(Some("FW: hello")).as_deref(), Some("hello"));
}

#[test]
fn mixed_prefixes() {
    assert_eq!(normalise_subject(Some("Re: Fwd: hello")).as_deref(), Some("hello"));
    assert_eq!(normalise_subject(Some("FW: Re: hello")).as_deref(), Some("hello"));
}

#[test]
fn collapses_internal_whitespace() {
    assert_eq!(normalise_subject(Some("hello\t  world")).as_deref(), Some("hello world"));
}

#[test]
fn case_insensitive_prefix_strip() {
    // Real-world clients sometimes lowercase "re:".
    assert_eq!(normalise_subject(Some("RE: hello")).as_deref(), Some("hello"));
    assert_eq!(normalise_subject(Some("re: hello")).as_deref(), Some("hello"));
}

#[test]
fn empty_and_none_return_none() {
    assert!(normalise_subject(None).is_none());
    assert!(normalise_subject(Some("")).is_none());
    assert!(normalise_subject(Some("   ")).is_none());
    assert!(normalise_subject(Some("Re: ")).is_none());
}
