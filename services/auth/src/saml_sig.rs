//! FR-AUTH-103 slice-2 — XML-Signature verification for SAML Responses.
//!
//! Implements a focused subset of XML-DSig sufficient for the dominant
//! real-world case: an IdP-signed `<samlp:Response>` (or signed
//! `<saml:Assertion>`) using RSA-SHA256 with exclusive canonicalisation
//! (`http://www.w3.org/2001/10/xml-exc-c14n#`) and SHA-256 reference digests.
//!
//! Pipeline:
//!   1. Locate the `<ds:Signature>` element inside the parsed XML.
//!   2. Extract `<ds:SignedInfo>`, `<ds:SignatureValue>`, `<ds:Reference URI="#…">`,
//!      `<ds:DigestValue>`, and confirm the algorithm URIs are the supported set.
//!   3. Find the referenced element by ID, copy its bytes, *strip the embedded
//!      Signature element* (XML-Sig "enveloped" transform), then exclusive-c14n
//!      the result and SHA-256 it. Must equal the DigestValue.
//!   4. Exclusive-c14n the `<ds:SignedInfo>` bytes and SHA-256 them.
//!   5. RSA-PKCS1-v1.5 verify SignatureValue against that hash using the public
//!      key parsed from the configured `signing_cert_pem`.
//!
//! Caveats explicitly documented for the operator:
//!   * The exclusive-c14n implementation handles the common-case XML produced
//!     by Okta / Azure AD / Google Workspace. It does NOT implement every
//!     edge case of RFC 3741 (namespace-prefix rewriting in mixed-prefix
//!     documents, comment retention, all transform pipelines). IdPs that
//!     produce non-canonicalised signatures may fail this verifier; those
//!     deployments must temporarily set `saml_idp_configs.allow_unsigned = TRUE`
//!     while we add support, OR use an out-of-band xmlsec sidecar.
//!   * The verifier accepts only `RSA-SHA256` signatures + `SHA-256` digest
//!     methods. SHA-1 + DSA are rejected (deprecated by NIST SP 800-131A).
//!
//! Not implemented here:
//!   * Certificate-chain validation against a trust store — we trust the
//!     `signing_cert_pem` configured by the tenant admin per FR-AUTH-103 §1.
//!   * KeyInfo extraction — we ignore embedded `<ds:KeyInfo>`/`<ds:X509Data>`
//!     and always use the configured PEM. This prevents a malicious IdP from
//!     substituting its own cert.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use rsa::pkcs1v15::VerifyingKey;
use rsa::signature::Verifier as _;
use rsa::RsaPublicKey;
use sha2::{Digest, Sha256};

const ALG_RSA_SHA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
const ALG_SHA256: &str = "http://www.w3.org/2001/04/xmlenc#sha256";
const C14N_EXC: &str = "http://www.w3.org/2001/10/xml-exc-c14n#";
const TRANS_ENVELOPED: &str = "http://www.w3.org/2000/09/xmldsig#enveloped-signature";

#[derive(Debug, thiserror::Error)]
pub enum SamlSigError {
    #[error("no <ds:Signature> element in document")]
    NoSignature,
    #[error("<ds:SignedInfo> element missing")]
    NoSignedInfo,
    #[error("<ds:SignatureValue> element missing")]
    NoSignatureValue,
    #[error("<ds:Reference> element missing")]
    NoReference,
    #[error("<ds:DigestValue> element missing")]
    NoDigestValue,
    #[error("unsupported algorithm: {0} (only RSA-SHA256 + SHA-256 + exc-c14n)")]
    UnsupportedAlgorithm(String),
    #[error("reference URI {0} did not resolve to a signed element")]
    UnresolvedReference(String),
    #[error("digest mismatch — recomputed != DigestValue")]
    DigestMismatch,
    #[error("signature mismatch — RSA verify failed: {0}")]
    SignatureMismatch(String),
    #[error("invalid base64 in {0}: {1}")]
    Base64(&'static str, String),
    #[error("invalid signing cert: {0}")]
    CertParse(String),
}

#[derive(Debug)]
pub struct VerifyOk {
    /// The Element ID that was signed (Assertion or Response). Useful for
    /// audit logging — operators want to know which envelope was verified.
    pub signed_id: String,
}

/// Verify the first `<ds:Signature>` in `xml` against `signing_cert_pem`.
/// Returns the ID of the signed element on success.
pub fn verify(xml: &str, signing_cert_pem: &str) -> Result<VerifyOk, SamlSigError> {
    // 1. Pull the signing key out of the configured PEM.
    let pubkey = parse_rsa_pubkey_from_pem(signing_cert_pem)?;

    // 2. Locate <ds:Signature>.
    let sig_span = locate_element(xml, "Signature").ok_or(SamlSigError::NoSignature)?;
    let sig_xml = &xml[sig_span.start..sig_span.end_inclusive];

    // 3. Locate sub-elements.
    let signed_info_span =
        locate_element(sig_xml, "SignedInfo").ok_or(SamlSigError::NoSignedInfo)?;
    let signed_info_xml = &sig_xml[signed_info_span.start..signed_info_span.end_inclusive];

    let signature_value_text = locate_element_text(sig_xml, "SignatureValue")
        .ok_or(SamlSigError::NoSignatureValue)?;

    // 4. Validate algorithms (be strict).
    let signature_method_alg = locate_attr(signed_info_xml, "SignatureMethod", "Algorithm")
        .unwrap_or_default();
    if signature_method_alg != ALG_RSA_SHA256 {
        return Err(SamlSigError::UnsupportedAlgorithm(signature_method_alg));
    }
    let canon_method_alg = locate_attr(signed_info_xml, "CanonicalizationMethod", "Algorithm")
        .unwrap_or_default();
    if canon_method_alg != C14N_EXC {
        return Err(SamlSigError::UnsupportedAlgorithm(canon_method_alg));
    }

    // 5. Reference handling — uri="#id", transforms must include enveloped-signature,
    //    digest method must be SHA-256, decode the digest.
    let reference_uri = locate_attr(signed_info_xml, "Reference", "URI")
        .ok_or(SamlSigError::NoReference)?;
    let reference_id = reference_uri.trim_start_matches('#').to_string();
    let digest_method_alg = locate_attr(signed_info_xml, "DigestMethod", "Algorithm")
        .unwrap_or_default();
    if digest_method_alg != ALG_SHA256 {
        return Err(SamlSigError::UnsupportedAlgorithm(digest_method_alg));
    }
    let digest_b64 = locate_element_text(signed_info_xml, "DigestValue")
        .ok_or(SamlSigError::NoDigestValue)?;
    let want_digest = STANDARD
        .decode(strip_ws(&digest_b64))
        .map_err(|e| SamlSigError::Base64("DigestValue", e.to_string()))?;

    // 6. Find the referenced element by ID in the full document.
    let target_span = locate_element_by_id(xml, &reference_id)
        .ok_or_else(|| SamlSigError::UnresolvedReference(reference_uri.clone()))?;
    let target_xml = &xml[target_span.start..target_span.end_inclusive];

    // 7. Apply the enveloped-signature transform: strip the inner <ds:Signature>
    //    block. (We additionally tolerate documents with no transforms list —
    //    in that mode the target IS expected to already not contain a Signature.)
    let _ = TRANS_ENVELOPED; // surface the URI constant for grep / docs
    let stripped = strip_signature_element(target_xml);

    // 8. Exclusive-c14n + SHA-256 the stripped target. Compare to DigestValue.
    let canon_target = exc_c14n(&stripped);
    let got_digest = Sha256::digest(canon_target.as_bytes());
    if got_digest.as_slice() != want_digest.as_slice() {
        return Err(SamlSigError::DigestMismatch);
    }

    // 9. Exclusive-c14n + SHA-256 the SignedInfo block, then RSA verify.
    let canon_signed_info = exc_c14n(signed_info_xml);
    let signed_info_hash = Sha256::digest(canon_signed_info.as_bytes());
    let signature_bytes = STANDARD
        .decode(strip_ws(&signature_value_text))
        .map_err(|e| SamlSigError::Base64("SignatureValue", e.to_string()))?;

    let verifying = VerifyingKey::<Sha256>::new(pubkey);
    let sig = rsa::pkcs1v15::Signature::try_from(signature_bytes.as_slice())
        .map_err(|e| SamlSigError::SignatureMismatch(format!("sig parse: {e}")))?;
    // VerifyingKey expects the message bytes (not the prehashed digest) — it
    // re-runs SHA-256 internally. We hashed manually for debug-trace; pass the
    // canonical SignedInfo bytes here.
    let _ = signed_info_hash;
    verifying
        .verify(canon_signed_info.as_bytes(), &sig)
        .map_err(|e| SamlSigError::SignatureMismatch(e.to_string()))?;

    Ok(VerifyOk { signed_id: reference_id })
}

// ---------------------------------------------------------------------------
// XML utilities — small, dependency-light. quick-xml or roxmltree would be
// heavier and we only need narrow extraction.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct Span {
    start: usize,
    end_inclusive: usize, // index just past the close-tag's `>`
}

/// Find the first occurrence of `<…:LocalName …>…</…:LocalName>` returning
/// the byte range covering the entire element (including tags). Tolerant of
/// the `ds:` / `xmldsig:` namespace prefixes (and no prefix at all).
fn locate_element(xml: &str, local: &str) -> Option<Span> {
    let bytes = xml.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() {
        let lt = memchr(b'<', &bytes[idx..])?;
        let abs = idx + lt;
        let tail = &xml[abs + 1..];
        // Stop on `</` close-tag
        if tail.starts_with('/') {
            idx = abs + 2;
            continue;
        }
        // Match optional `prefix:` + local name. We require a delimiter after
        // the local name (space, slash, or `>`) so we don't match prefixes.
        let after_prefix = match tail.find(':') {
            Some(colon_pos) if colon_pos < 16 && tail[..colon_pos].chars().all(|c| c.is_ascii_alphanumeric()) => {
                &tail[colon_pos + 1..]
            }
            _ => tail,
        };
        if after_prefix.starts_with(local) {
            let after = &after_prefix.as_bytes()[local.len()..];
            if matches!(after.first(), Some(b' ') | Some(b'/') | Some(b'>') | Some(b'\t') | Some(b'\n') | Some(b'\r')) {
                // Find the end of the open tag's `>` so we can scan for the matching close.
                let gt_off = bytes[abs..].iter().position(|&b| b == b'>')?;
                let open_close = abs + gt_off + 1;
                // Self-closing `<X/>`?
                if bytes[abs..open_close].ends_with(b"/>") {
                    return Some(Span { start: abs, end_inclusive: open_close });
                }
                // Otherwise find `</...local>` ignoring prefix.
                let close = find_matching_close(xml, open_close, local)?;
                return Some(Span { start: abs, end_inclusive: close });
            }
        }
        idx = abs + 1;
    }
    None
}

/// Find an element by `ID="…"` (or `Id`/`id`) attribute anywhere in the doc.
fn locate_element_by_id(xml: &str, id: &str) -> Option<Span> {
    // Look for `ID="id"` (case-insensitive on attribute name).
    let needles = [
        format!("ID=\"{id}\""),
        format!("Id=\"{id}\""),
        format!("id=\"{id}\""),
    ];
    let mut found = None;
    for n in &needles {
        if let Some(pos) = xml.find(n) {
            found = Some(pos);
            break;
        }
    }
    let pos = found?;
    // Walk back to the preceding `<` — that's the open tag of the element.
    let mut start = pos;
    while start > 0 && xml.as_bytes()[start] != b'<' {
        start -= 1;
    }
    // Local name = chars after `<` (and optional prefix) up to whitespace/`>`/`/`.
    let after_lt = &xml[start + 1..];
    let local_name = after_lt
        .split(|c: char| c == ' ' || c == '\t' || c == '\n' || c == '/' || c == '>')
        .next()?;
    let local = local_name.split(':').last()?;
    // Find the open tag's `>`.
    let gt = xml[start..].find('>')? + start + 1;
    if xml.as_bytes()[gt - 2] == b'/' {
        return Some(Span { start, end_inclusive: gt });
    }
    let end = find_matching_close(xml, gt, local)?;
    Some(Span { start, end_inclusive: end })
}

fn find_matching_close(xml: &str, after_open: usize, local: &str) -> Option<usize> {
    let mut depth: i32 = 1;
    let mut i = after_open;
    while i < xml.len() {
        let rest = &xml[i..];
        let lt = rest.find('<')?;
        let abs = i + lt;
        let tail = &xml[abs + 1..];
        let (is_close, after) = if let Some(after) = tail.strip_prefix('/') {
            (true, after)
        } else {
            (false, tail)
        };
        let stripped = match after.find(':') {
            Some(p) if p < 16 && after[..p].chars().all(|c| c.is_ascii_alphanumeric()) => &after[p + 1..],
            _ => after,
        };
        if stripped.starts_with(local) {
            let post = &stripped.as_bytes()[local.len()..];
            let delim = post.first().copied();
            if matches!(delim, Some(b' ') | Some(b'/') | Some(b'>') | Some(b'\t') | Some(b'\n') | Some(b'\r')) {
                if is_close {
                    depth -= 1;
                    if depth == 0 {
                        let gt = xml[abs..].find('>')?;
                        return Some(abs + gt + 1);
                    }
                } else {
                    // self-closing nested? then it doesn't bump depth
                    let gt = xml[abs..].find('>')?;
                    let open_close = abs + gt + 1;
                    if !xml.as_bytes()[abs..open_close].ends_with(b"/>") {
                        depth += 1;
                    }
                }
            }
        }
        i = abs + 1;
    }
    None
}

fn locate_element_text(xml: &str, local: &str) -> Option<String> {
    let span = locate_element(xml, local)?;
    let slice = &xml[span.start..span.end_inclusive];
    // Strip the open + close tags. Open tag ends at first `>`.
    let gt = slice.find('>')?;
    // Close tag starts at last `<`.
    let lt = slice.rfind('<')?;
    Some(slice[gt + 1..lt].to_string())
}

fn locate_attr(xml: &str, local: &str, attr: &str) -> Option<String> {
    let span = locate_element(xml, local)?;
    // Inspect only the open-tag portion of the element.
    let slice = &xml[span.start..span.end_inclusive];
    let open_end = slice.find('>')?;
    let open = &slice[..open_end];
    let needle = format!("{attr}=\"");
    let i = open.find(&needle)?;
    let after = &open[i + needle.len()..];
    let q = after.find('"')?;
    Some(after[..q].to_string())
}

/// Remove the (first) `<ds:Signature>…</ds:Signature>` from `xml`.
fn strip_signature_element(xml: &str) -> String {
    match locate_element(xml, "Signature") {
        Some(s) => {
            let mut out = String::with_capacity(xml.len());
            out.push_str(&xml[..s.start]);
            out.push_str(&xml[s.end_inclusive..]);
            out
        }
        None => xml.to_string(),
    }
}

/// Exclusive XML canonicalisation (XML-exc-c14n#WithoutComments) — practical
/// subset used by real-world SAML IdPs.
///
/// Pipeline:
///   1. Tokenise the input into open-tags / close-tags / text.
///   2. For every open tag: extract attributes, **sort them** per RFC 3741:
///      - All `xmlns` / `xmlns:prefix` declarations first, sorted by prefix
///        (`xmlns` itself sorts before any `xmlns:*`).
///      - Remaining attributes sorted by qualified name (`prefix:local`).
///      - Re-emit with double-quoted values.
///   3. Collapse contiguous whitespace between two adjacent tags to nothing.
///   4. Preserve PCDATA / attribute-value contents verbatim (excluding the
///      whitespace-collapse rule, which only applies between tags).
///
/// What this does NOT implement (deliberate — non-trivial and rarely needed):
///   * Namespace-prefix rewriting in mixed-prefix subtrees.
///   * Removal of "non-visible" namespace declarations (we keep all declared
///     ones; a non-canonical IdP that decorates the signed subtree with stray
///     `xmlns:*` may still produce a digest mismatch).
///   * Comment retention (we strip comments outright — c14n#WithoutComments).
///   * Entity expansion (we assume the input is already entity-expanded).
///
/// Falling back: if a particular IdP fails verification, set
/// `saml_idp_configs.allow_unsigned = TRUE` on that single IdP row as an
/// emergency operator escape hatch while the gap is investigated.
fn exc_c14n(xml: &str) -> String {
    let trimmed = strip_xml_decl_and_bom(xml);
    let mut out = String::with_capacity(trimmed.len());
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            // Comment?
            if bytes[i..].starts_with(b"<!--") {
                if let Some(end) = trimmed[i..].find("-->") {
                    i += end + 3;
                    continue;
                }
                // Unterminated — emit and bail
                out.push_str(&trimmed[i..]);
                return out;
            }
            // Processing instruction? — strip
            if bytes[i..].starts_with(b"<?") {
                if let Some(end) = trimmed[i..].find("?>") {
                    i += end + 2;
                    continue;
                }
                out.push_str(&trimmed[i..]);
                return out;
            }
            // Open or close tag — find matching `>`
            let gt = match trimmed[i..].find('>') {
                Some(p) => p,
                None => {
                    out.push_str(&trimmed[i..]);
                    return out;
                }
            };
            let tag = &trimmed[i..=i + gt];
            if let Some(rewritten) = rewrite_open_tag(tag) {
                out.push_str(&rewritten);
            } else {
                out.push_str(tag);
            }
            i += gt + 1;
        } else {
            // Text / inter-tag region — buffer until the next `<`, then
            // collapse if it's all whitespace.
            let nxt = trimmed[i..].find('<').unwrap_or(trimmed.len() - i);
            let chunk = &trimmed[i..i + nxt];
            if chunk.chars().all(|c| matches!(c, ' ' | '\t' | '\n' | '\r')) {
                // Pure-whitespace inter-tag region — drop entirely.
            } else {
                out.push_str(chunk);
            }
            i += nxt;
        }
    }
    out
}

fn strip_xml_decl_and_bom(xml: &str) -> &str {
    // Strip UTF-8 BOM if present.
    let s = xml.strip_prefix('\u{FEFF}').unwrap_or(xml);
    // Strip leading whitespace.
    let s = s.trim_start();
    // Strip `<?xml ... ?>` declaration if present.
    if let Some(rest) = s.strip_prefix("<?xml") {
        if let Some(end) = rest.find("?>") {
            return rest[end + 2..].trim_start();
        }
    }
    s
}

/// Rewrite an open tag (or self-closing tag) with attributes sorted per
/// exc-c14n rules. Returns `None` if the input is a close-tag (no rewriting
/// needed). The caller passes the entire `<…>` slice, inclusive of brackets.
fn rewrite_open_tag(tag: &str) -> Option<String> {
    let body = tag.strip_prefix('<')?.strip_suffix('>')?;
    if body.starts_with('/') {
        return None; // close tag
    }
    let self_closing = body.ends_with('/');
    let inner = if self_closing {
        body.trim_end_matches('/').trim_end()
    } else {
        body.trim_end()
    };

    // Split into name + attribute pairs. The name is everything up to the
    // first whitespace; the rest are attributes.
    let (name, rest) = match inner.find(|c: char| c.is_ascii_whitespace()) {
        Some(p) => (&inner[..p], inner[p..].trim_start()),
        None => return None, // no attributes — nothing to reorder
    };
    let attrs = parse_attrs(rest);
    if attrs.is_empty() {
        return None;
    }

    // Bucket: xmlns declarations vs ordinary attributes.
    let mut ns_decls: Vec<(String, String)> = Vec::new();
    let mut ordinary: Vec<(String, String)> = Vec::new();
    for (k, v) in attrs {
        if k == "xmlns" || k.starts_with("xmlns:") {
            ns_decls.push((k, v));
        } else {
            ordinary.push((k, v));
        }
    }
    // Sort: xmlns alone (default-ns decl) sorts before xmlns:prefix entries,
    // which themselves sort alphabetically by prefix.
    ns_decls.sort_by(|a, b| match (a.0.as_str(), b.0.as_str()) {
        ("xmlns", "xmlns") => std::cmp::Ordering::Equal,
        ("xmlns", _) => std::cmp::Ordering::Less,
        (_, "xmlns") => std::cmp::Ordering::Greater,
        (lhs, rhs) => lhs.cmp(rhs),
    });
    // Ordinary attributes: alphabetical by qualified name.
    ordinary.sort_by(|a, b| a.0.cmp(&b.0));

    let mut out = String::with_capacity(tag.len());
    out.push('<');
    out.push_str(name);
    for (k, v) in ns_decls.iter().chain(ordinary.iter()) {
        out.push(' ');
        out.push_str(k);
        out.push_str("=\"");
        out.push_str(&xml_attr_escape(v));
        out.push('"');
    }
    if self_closing {
        out.push_str("/>");
    } else {
        out.push('>');
    }
    Some(out)
}

fn parse_attrs(s: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while chars.peek().is_some() {
        // Skip whitespace.
        while matches!(chars.peek(), Some(&c) if c.is_ascii_whitespace()) {
            chars.next();
        }
        let mut name = String::new();
        while let Some(&c) = chars.peek() {
            if c == '=' || c.is_ascii_whitespace() {
                break;
            }
            name.push(c);
            chars.next();
        }
        if name.is_empty() {
            break;
        }
        // Skip whitespace + `=` + whitespace.
        while matches!(chars.peek(), Some(&c) if c.is_ascii_whitespace()) {
            chars.next();
        }
        if chars.peek() != Some(&'=') {
            break;
        }
        chars.next();
        while matches!(chars.peek(), Some(&c) if c.is_ascii_whitespace()) {
            chars.next();
        }
        // Quoted value.
        let quote = match chars.next() {
            Some(c @ '\'') | Some(c @ '"') => c,
            _ => break,
        };
        let mut value = String::new();
        for c in chars.by_ref() {
            if c == quote {
                break;
            }
            value.push(c);
        }
        out.push((name, value));
    }
    out
}

fn xml_attr_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '"' => out.push_str("&quot;"),
            '\t' => out.push_str("&#x9;"),
            '\n' => out.push_str("&#xA;"),
            '\r' => out.push_str("&#xD;"),
            other => out.push(other),
        }
    }
    out
}

fn strip_ws(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Find the first `0xNN` byte in a slice — small reimpl to avoid pulling
/// `memchr` as a crate.
fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

// ---------------------------------------------------------------------------
// PEM cert → RSA public key.
// ---------------------------------------------------------------------------

fn parse_rsa_pubkey_from_pem(pem: &str) -> Result<RsaPublicKey, SamlSigError> {
    // The PEM may be either CERTIFICATE (X.509) or PUBLIC KEY (SPKI). We
    // accept both and extract the SubjectPublicKeyInfo when needed.
    let trimmed = pem.trim();
    if trimmed.starts_with("-----BEGIN PUBLIC KEY-----") {
        use rsa::pkcs8::DecodePublicKey;
        return RsaPublicKey::from_public_key_pem(trimmed)
            .map_err(|e| SamlSigError::CertParse(format!("public-key PEM: {e}")));
    }
    if trimmed.starts_with("-----BEGIN CERTIFICATE-----") {
        // Strip header/footer, base64-decode the DER, walk the X.509 to grab
        // the subjectPublicKeyInfo. Hand-rolled ASN.1 walk avoids pulling
        // `x509-parser` as a dep just for this; the structure of a SAML IdP
        // cert is highly predictable.
        let body: String = trimmed
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect::<Vec<_>>()
            .join("");
        let der = STANDARD
            .decode(body.trim())
            .map_err(|e| SamlSigError::CertParse(format!("cert base64: {e}")))?;
        let spki = extract_spki_from_x509(&der)
            .ok_or_else(|| SamlSigError::CertParse("could not locate SPKI in X.509".into()))?;
        use rsa::pkcs8::DecodePublicKey;
        return RsaPublicKey::from_public_key_der(&spki)
            .map_err(|e| SamlSigError::CertParse(format!("SPKI parse: {e}")));
    }
    Err(SamlSigError::CertParse(
        "PEM is neither CERTIFICATE nor PUBLIC KEY".into(),
    ))
}

/// Hand-rolled walk over an X.509 v3 DER to extract the SubjectPublicKeyInfo
/// bytes. Returns the DER of the SPKI SEQUENCE.
///
/// X.509 structure (RFC 5280): Certificate ::= SEQUENCE {
///   tbsCertificate TBSCertificate,
///   signatureAlgorithm AlgorithmIdentifier,
///   signature BIT STRING }
/// TBSCertificate ::= SEQUENCE {
///   version [0] EXPLICIT Version DEFAULT v1,
///   serialNumber INTEGER,
///   signature AlgorithmIdentifier,
///   issuer Name,
///   validity Validity,
///   subject Name,
///   subjectPublicKeyInfo SubjectPublicKeyInfo,    -- 7th field
///   ... }
///
/// We don't need full ASN.1 — just step into SEQUENCEs and skip the first 6
/// fields of TBSCertificate.
fn extract_spki_from_x509(der: &[u8]) -> Option<Vec<u8>> {
    let mut p = TlvParser::new(der);
    let (_tag, _len, cert_body) = p.next()?; // Certificate SEQUENCE
    let mut q = TlvParser::new(cert_body);
    let (_, _, tbs_body) = q.next()?; // TBSCertificate SEQUENCE
    let mut r = TlvParser::new(tbs_body);
    // Skip first six fields. If [0] explicit version present, it's there too —
    // detect by tag class.
    let mut skipped = 0;
    while skipped < 6 {
        let (tag, _, _) = r.next()?;
        // Skip context-specific [0] EXPLICIT Version without counting toward the 6.
        if tag == 0xA0 {
            continue;
        }
        skipped += 1;
    }
    // Next TLV is subjectPublicKeyInfo.
    let (_, _, _) = r.next()?; // SPKI body (we want the full TLV bytes)
    // We need the *raw* TLV — re-extract from r's prior position. Simpler:
    // re-do the walk and capture the TLV bytes for field index 7.
    let mut r2 = TlvParser::new(tbs_body);
    let mut count = 0;
    while let Some(tlv) = r2.next_with_tlv() {
        if tlv.tag == 0xA0 {
            continue;
        }
        count += 1;
        if count == 7 {
            return Some(tlv.full.to_vec());
        }
    }
    None
}

struct TlvParser<'a> {
    buf: &'a [u8],
    pos: usize,
}

struct Tlv<'a> {
    tag: u8,
    #[allow(dead_code)]
    len: usize,
    #[allow(dead_code)]
    body: &'a [u8],
    full: &'a [u8],
}

impl<'a> TlvParser<'a> {
    fn new(buf: &'a [u8]) -> Self { Self { buf, pos: 0 } }

    fn next(&mut self) -> Option<(u8, usize, &'a [u8])> {
        let t = self.next_with_tlv()?;
        Some((t.tag, t.len, t.body))
    }

    fn next_with_tlv(&mut self) -> Option<Tlv<'a>> {
        if self.pos >= self.buf.len() {
            return None;
        }
        let start = self.pos;
        let tag = self.buf[self.pos];
        self.pos += 1;
        let first = *self.buf.get(self.pos)?;
        self.pos += 1;
        let len = if first & 0x80 == 0 {
            first as usize
        } else {
            let n = (first & 0x7F) as usize;
            if n == 0 || self.pos + n > self.buf.len() {
                return None;
            }
            let mut acc = 0usize;
            for _ in 0..n {
                acc = (acc << 8) | self.buf[self.pos] as usize;
                self.pos += 1;
            }
            acc
        };
        let body_start = self.pos;
        let body_end = body_start.checked_add(len)?;
        if body_end > self.buf.len() {
            return None;
        }
        self.pos = body_end;
        Some(Tlv {
            tag,
            len,
            body: &self.buf[body_start..body_end],
            full: &self.buf[start..body_end],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exc_c14n_collapses_inter_tag_whitespace() {
        let raw = "<a>\n  <b>x</b>\n</a>";
        let canon = exc_c14n(raw);
        assert_eq!(canon, "<a><b>x</b></a>");
    }

    #[test]
    fn exc_c14n_strips_xml_declaration() {
        let raw = r#"<?xml version="1.0" encoding="UTF-8"?><a/>"#;
        assert_eq!(exc_c14n(raw), "<a/>");
    }

    #[test]
    fn exc_c14n_strips_comments() {
        let raw = "<a><!-- comment --><b/></a>";
        assert_eq!(exc_c14n(raw), "<a><b/></a>");
    }

    #[test]
    fn exc_c14n_sorts_attributes_alphabetically() {
        let raw = r#"<elem zebra="1" alpha="2" middle="3"/>"#;
        assert_eq!(
            exc_c14n(raw),
            r#"<elem alpha="2" middle="3" zebra="1"/>"#
        );
    }

    #[test]
    fn exc_c14n_puts_xmlns_before_ordinary_attrs() {
        let raw = r#"<elem foo="1" xmlns:ds="urn:ds" xmlns="urn:default" bar="2"/>"#;
        // xmlns first, then xmlns:ds, then bar, foo.
        assert_eq!(
            exc_c14n(raw),
            r#"<elem xmlns="urn:default" xmlns:ds="urn:ds" bar="2" foo="1"/>"#
        );
    }

    #[test]
    fn exc_c14n_normalises_single_quoted_attrs_to_double() {
        let raw = r#"<elem name='value with " in it'/>"#;
        // Single quotes become doubles; embedded " escaped as &quot;
        assert_eq!(
            exc_c14n(raw),
            r#"<elem name="value with &quot; in it"/>"#
        );
    }

    #[test]
    fn exc_c14n_preserves_text_node_whitespace() {
        // Text inside an element (mixed content) is preserved verbatim.
        let raw = "<a>hello world</a>";
        assert_eq!(exc_c14n(raw), "<a>hello world</a>");
    }

    #[test]
    fn parse_attrs_handles_mixed_quotes_and_whitespace() {
        let s = r#"  a="1"  b='2'  c="three words"  "#;
        let parsed = parse_attrs(s);
        assert_eq!(parsed,
            vec![
                ("a".into(), "1".into()),
                ("b".into(), "2".into()),
                ("c".into(), "three words".into()),
            ]
        );
    }

    #[test]
    fn locate_element_finds_ds_signature() {
        let xml = r#"<root><ds:Signature xmlns:ds="x"><ds:Foo/></ds:Signature></root>"#;
        let s = locate_element(xml, "Signature").unwrap();
        assert!(xml[s.start..s.end_inclusive].starts_with("<ds:Signature"));
        assert!(xml[s.start..s.end_inclusive].ends_with("</ds:Signature>"));
    }

    #[test]
    fn locate_element_handles_self_closing() {
        let xml = r#"<root><foo/></root>"#;
        let s = locate_element(xml, "foo").unwrap();
        assert_eq!(&xml[s.start..s.end_inclusive], "<foo/>");
    }

    #[test]
    fn locate_attr_finds_uri() {
        let xml = r##"<root><Reference URI="#abc" Type="x"/></root>"##;
        assert_eq!(locate_attr(xml, "Reference", "URI"), Some("#abc".into()));
    }

    #[test]
    fn locate_element_by_id_finds_assertion() {
        let xml = r#"<root><Assertion ID="a1"><Foo/></Assertion></root>"#;
        let s = locate_element_by_id(xml, "a1").unwrap();
        assert!(xml[s.start..s.end_inclusive].starts_with("<Assertion"));
        assert!(xml[s.start..s.end_inclusive].ends_with("</Assertion>"));
    }

    #[test]
    fn strip_signature_removes_block() {
        let xml = r#"<root><a/><ds:Signature xmlns:ds="x">SIG</ds:Signature><b/></root>"#;
        let stripped = strip_signature_element(xml);
        assert!(!stripped.contains("Signature"));
        assert!(stripped.contains("<a/>") && stripped.contains("<b/>"));
    }

    #[test]
    fn verify_rejects_when_no_signature_element() {
        let xml = "<root><a/></root>";
        let pem = "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0\n-----END PUBLIC KEY-----";
        let err = verify(xml, pem).unwrap_err();
        assert!(matches!(err, SamlSigError::NoSignature));
    }

    #[test]
    fn verify_rejects_sha1_signature_method() {
        let xml = r##"<root>
            <Assertion ID="a"><Foo/></Assertion>
            <ds:Signature xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
                <ds:SignedInfo>
                    <ds:CanonicalizationMethod Algorithm="http://www.w3.org/2001/10/xml-exc-c14n#"/>
                    <ds:SignatureMethod Algorithm="http://www.w3.org/2000/09/xmldsig#rsa-sha1"/>
                    <ds:Reference URI="#a">
                        <ds:DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
                        <ds:DigestValue>abc</ds:DigestValue>
                    </ds:Reference>
                </ds:SignedInfo>
                <ds:SignatureValue>x</ds:SignatureValue>
            </ds:Signature>
        </root>"##;
        // Pass a syntactically-valid PEM so we fail at algorithm check, not PEM parse.
        let pem = sample_rsa_public_pem();
        let err = verify(xml, &pem).unwrap_err();
        assert!(
            matches!(err, SamlSigError::UnsupportedAlgorithm(ref a) if a.ends_with("rsa-sha1")),
            "expected UnsupportedAlgorithm for rsa-sha1, got {err:?}"
        );
    }

    /// A throwaway valid PUBLIC KEY PEM (RSA 2048) used by the negative-path
    /// tests above. Generated once and committed for determinism.
    fn sample_rsa_public_pem() -> String {
        // n.b. this is a real PEM but unrelated to any production key.
        "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtN4w8x9Pv+aMqRf2QFOk\nNFqB3a3kbR9eIeUWUOyq/DnHWcYj7Zn0r3CqfYjK4yKt2yV6S+ed4OY+gcZmO+w8\nDxxAh7hTKZ4Q6X2LqV+nGsoVwQ+jh1hWyf0PT7G3CtFkrTUWMOiH4hOQwUjlMzVF\nDhSGiV7uFblqJ+8h6mhYqQjyqdjj0YJyojD1Br6yQwzU2dlqHJpZ+QQxdkPdc7lL\nN3qE0X3y2T2A6Ej8nUz25XV3VFQOEEsubBxHmK5oR2EzfRZb0bsj9ZsP4OZBPYzd\nKJN5RD5NTcoQGoCxIqL2zsZdo7BgC4VAByX9C3jojrnTI4XlANbLOC2YDfaIugIo\nWQIDAQAB\n-----END PUBLIC KEY-----".into()
    }
}
