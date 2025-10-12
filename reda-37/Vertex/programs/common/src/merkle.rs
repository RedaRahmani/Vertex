//! Deterministic Merkle proof verification helpers.
//!
//! Notes on construction:
//! - Pairs are hashed in sorted order (lexicographically by 32-byte value).
//!   This means when folding the proof we order `(left, right)` such that
//!   the lower value is on the left before hashing.
//! - The leaf hashing scheme should be mirrored off-chain.
//!   For vesting, we hash `beneficiary_pubkey (32 bytes) || amount (u64 LE)` with keccak256.
//!   Off-chain generators must use the exact same byte order to match.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;

/// Computes keccak256 hash of concatenated nodes.
pub fn hash_nodes(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut data = [0u8; 64];
    data[..32].copy_from_slice(left);
    data[32..].copy_from_slice(right);
    keccak::hash(&data).to_bytes()
}

/// Verifies Merkle proof for given leaf and root.
pub fn verify_merkle_proof(leaf: [u8; 32], proof: &[[u8; 32]], root: [u8; 32]) -> bool {
    proof.iter().fold(leaf, |acc, node| {
        if acc <= *node {
            hash_nodes(&acc, node)
        } else {
            hash_nodes(node, &acc)
        }
    }) == root
}

/// Anchor-compatible guard ensuring proof is valid.
pub fn assert_merkle_proof(leaf: [u8; 32], proof: &[[u8; 32]], root: [u8; 32]) -> Result<()> {
    require!(
        verify_merkle_proof(leaf, proof, root),
        crate::errors::CommonError::InvalidMerkleProof
    );
    Ok(())
}
