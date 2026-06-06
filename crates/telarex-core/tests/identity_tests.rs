use telarex_core::network::auth::QuantumAuth;

#[test]
fn test_quantum_identity_generation() {
    let identity = QuantumAuth::generate_identity();
    assert!(!identity.0.is_empty(), "public key should not be empty");
    assert!(!identity.1.is_empty(), "secret key should not be empty");
}

#[test]
fn test_sign_and_verify_roundtrip() {
    let (pk, sk) = QuantumAuth::generate_identity();
    let challenge = b"authenticate-to-lodge-alpha-42";
    let signature = QuantumAuth::sign_challenge(challenge, &sk);
    assert!(!signature.is_empty(), "signature should not be empty");
    let result = QuantumAuth::verify(challenge, &signature, &pk);
    assert!(result, "verification should pass with correct key");
}

#[test]
fn test_verify_rejects_wrong_key() {
    let (pk, sk) = QuantumAuth::generate_identity();
    let (wrong_pk, _) = QuantumAuth::generate_identity();
    let challenge = b"test-challenge";
    let signature = QuantumAuth::sign_challenge(challenge, &sk);
    let result = QuantumAuth::verify(challenge, &signature, &wrong_pk);
    assert!(!result, "verification should fail with wrong public key");
}

#[test]
fn test_verify_rejects_wrong_message() {
    let (pk, sk) = QuantumAuth::generate_identity();
    let challenge = b"real-message";
    let signature = QuantumAuth::sign_challenge(challenge, &sk);
    let result = QuantumAuth::verify(b"fake-message", &signature, &pk);
    assert!(!result, "verification should fail with wrong message");
}

#[test]
fn test_signature_is_deterministic() {
    let (pk, sk) = QuantumAuth::generate_identity();
    let challenge = b"deterministic-test";
    let sig1 = QuantumAuth::sign_challenge(challenge, &sk);
    let sig2 = QuantumAuth::sign_challenge(challenge, &sk);
    // ML-DSA is deterministic for the same message + sk
    assert_eq!(sig1, sig2, "signatures should be identical for same inputs");
    let _ = pk; // prevent unused warning
}
