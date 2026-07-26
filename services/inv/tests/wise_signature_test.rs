use cyberos_inv::wise::{verify_signature, WiseVerifyError};
use rand::rngs::OsRng;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::{EncodePublicKey, LineEnding};
use rsa::signature::{SignatureEncoding, Signer};
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;

fn keypair_pem() -> (String, SigningKey<Sha256>) {
    let mut rng = OsRng;
    let private = RsaPrivateKey::new(&mut rng, 2048).unwrap();
    let public = RsaPublicKey::from(&private);
    let pem = public.to_public_key_pem(LineEnding::LF).unwrap();
    let signing = SigningKey::<Sha256>::new(private);
    (pem, signing)
}

#[test]
fn valid_signature_accepts() {
    let (pem, signing) = keypair_pem();
    let body = br#"{"event_type":"balances#credit"}"#;
    let sig = signing.sign(body);
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, sig.to_bytes());
    verify_signature(&pem, body, &b64).unwrap();
}

#[test]
fn tampered_body_rejects() {
    let (pem, signing) = keypair_pem();
    let body = br#"{"event_type":"balances#credit"}"#;
    let sig = signing.sign(body);
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, sig.to_bytes());
    let err = verify_signature(&pem, br#"{"event_type":"tampered"}"#, &b64).unwrap_err();
    assert_eq!(err, WiseVerifyError::InvalidSignature);
}
