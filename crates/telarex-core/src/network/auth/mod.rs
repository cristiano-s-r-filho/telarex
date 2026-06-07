//! Authentication — quantum-resistant identity using ML-DSA (Dilithium).
//!
//! [`QuantumAuth`] provides static methods for generating keys, signing
//! challenges, and verifying proofs. [`QuantumId`] holds a peer's public key.
//! This replaces the classical Schnorr-based ZK identification scheme.

pub use pqc_dilithium::Keypair;
use serde::{Deserialize, Serialize};

/// Quantum-resistant identity system using ML-DSA (Dilithium).
///
/// Provides static methods for key generation, challenge signing, and proof
/// verification. This replaces the earlier Schnorr-based ZK scheme.
pub struct QuantumAuth;

/// A quantum-safe identity containing only the public key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumId {
    pub public_key: Vec<u8>,
}

impl QuantumAuth {
    /// Generates a new Quantum-safe identity (Public/Private keypair)
    pub fn generate_identity() -> Keypair {
        Keypair::generate()
    }

    /// Sign a challenge to prove identity
    pub fn sign_challenge(keys: &Keypair, challenge: &[u8]) -> Vec<u8> {
        keys.sign(challenge).to_vec()
    }

    /// Verify a proof of identity
    pub fn verify(public_key: &[u8], challenge: &[u8], proof: &[u8]) -> bool {
        pqc_dilithium::verify(proof, challenge, public_key).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_identity_loop() {
        let challenge = b"prove-you-are-user-123";
        let keys = Keypair::generate();
        let sig = keys.sign(challenge);
        assert!(QuantumAuth::verify(&keys.public, challenge, &sig));
    }
}
