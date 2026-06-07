use telarex_core::network::auth::{QuantumAuth, Keypair};

#[test]
fn test_quantum_identity_generation() {
    let keys = QuantumAuth::generate_identity();
    assert!(!keys.public.is_empty(), "public key should not be empty");
    assert!(!keys.expose_secret().is_empty(), "secret key should not be empty");
}

#[test]
fn test_sign_and_verify_roundtrip() {
    let keys = Keypair::generate();
    let challenge = b"authenticate-to-lodge-alpha-42";
    let signature = QuantumAuth::sign_challenge(&keys, challenge);
    assert!(!signature.is_empty(), "signature should not be empty");
    let result = QuantumAuth::verify(&keys.public, challenge, &signature);
    assert!(result, "verification should pass with correct key");
}

#[test]
fn test_verify_rejects_wrong_key() {
    let keys = Keypair::generate();
    let wrong_keys = Keypair::generate();
    let challenge = b"test-challenge";
    let signature = QuantumAuth::sign_challenge(&keys, challenge);
    let result = QuantumAuth::verify(&wrong_keys.public, challenge, &signature);
    assert!(!result, "verification should fail with wrong public key");
}

#[test]
fn test_verify_rejects_wrong_message() {
    let keys = Keypair::generate();
    let challenge = b"real-message";
    let signature = QuantumAuth::sign_challenge(&keys, challenge);
    let result = QuantumAuth::verify(&keys.public, b"fake-message", &signature);
    assert!(!result, "verification should fail with wrong message");
}

#[test]
fn test_signature_is_deterministic() {
    let keys = Keypair::generate();
    let challenge = b"deterministic-test";
    let sig1 = QuantumAuth::sign_challenge(&keys, challenge);
    let sig2 = QuantumAuth::sign_challenge(&keys, challenge);
    // ML-DSA is deterministic for the same message + sk
    assert_eq!(sig1, sig2, "signatures should be identical for same inputs");
}
