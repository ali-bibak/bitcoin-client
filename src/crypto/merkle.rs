use ring::digest::{digest, SHA256};
use std::borrow::Borrow;
use serde::{Serialize, Deserialize};

use super::hash::{Hashable, H256};

/// A node in the Merkle tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode{
    key: H256,
    left_child: Box<Option<MerkleNode>>,
    right_child: Box<Option<MerkleNode>>,
}

/// A Merkle tree.
#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleTree {
    root: MerkleNode,
}

/// Build a Merkle tree from a set of leaves (recursively)
fn build(leaves: Vec<MerkleNode>, leaf_size: usize) -> MerkleNode {
    let mut n = leaf_size;
    if n == 1 {
        let root = leaves[0].clone();
        return root;
    }
    let mut flag = false;
    if n % 2 == 1 {
        n += 1;
        flag = true;
    }
    n = n / 2;
    let mut new_leaves: Vec<MerkleNode> = Vec::new();
    for i in 0..n {
        let elem1: MerkleNode = leaves[2 * i].clone();
        let elem2: MerkleNode = match flag && i == n - 1 {
            true => leaves[2 * i].clone(),
            false => leaves[2 * i + 1].clone(),
        };
        let hash1 = (elem1.key).as_ref();
        let hash2 = (elem2.key).as_ref();
        let concat_hash = H256::from(digest(&SHA256, &[hash1, hash2].concat()));
        let par: MerkleNode = MerkleNode {
            key: concat_hash,
            left_child: Box::new(Option::from(elem1)),
            right_child: Box::new(Option::from(elem2)),
        };
        new_leaves.push(par);
    }
    let root = build(new_leaves, n);
    return root;
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable {
        let leaf_size = data.len();
        let mut leaves: Vec<MerkleNode> = Vec::new();
        for i in 0..leaf_size {
            let dt = data[i].borrow();
            let hashed = Hashable::hash(dt);
            let elem: MerkleNode = MerkleNode {
                key: hashed,
                left_child: Box::new(None),
                right_child: Box::new(None),
            };
            leaves.push(elem);
        }
        let root = build(leaves, leaf_size);
        let tree: MerkleTree = MerkleTree {
            root,
        };
        return tree;
    }

    pub fn root(&self) -> H256 {
        let r = self.root.clone();
        let h = r.key;
        return h;
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut binary: Vec<usize> = Vec::new();
        let mut n = index;
        while {
            binary.push(n % 2);
            n /= 2;
            n != 0
        } {}
        let m = binary.len();
        let mut current = self.root.clone();
        let mut proof_vec: Vec<H256> = Vec::new();
        for i in 0..m {
            let lc = current.left_child.unwrap();
            let rc = current.right_child.unwrap();
            if binary[i] == 0 {
                proof_vec.push(rc.key);
                current = lc;
            } else {
                proof_vec.push(lc.key);
                current = rc;
            }
        }
        return proof_vec;
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    let m = proof.len();
    let mut n = leaf_size;
    let mut i = index;
    let mut j = 1;
    let mut current = datum.clone();
    while n > 1 && j <= m {
        if i % 2 == 0 {
            let concat = [current.as_ref(), proof[m - j].as_ref()].concat();
            let hashed = digest(&SHA256, &concat);
            let concat_hash = H256::from(hashed);
            current = concat_hash;
        } else {
            let concat = [proof[m - j].as_ref(), current.as_ref()].concat();
            let hashed = digest(&SHA256, &concat);
            let concat_hash = H256::from(hashed);
            current = concat_hash;
        }
        n = n / 2;
        i = i / 2;
        j = j + 1;
    }
    if n == 1 && j == m + 1 && current.eq(root) {
        return true;
    }
    return false;
}

#[cfg(test)]
mod tests {
    use crate::crypto::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
    () => {{
    vec![
    (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
    (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
    ]
    }};
    }

    #[test]
    fn root() {
    let input_data: Vec<H256> = gen_merkle_tree_data!();
    let merkle_tree = MerkleTree::new(&input_data);
    let root = merkle_tree.root();
    assert_eq!(
    root,
    (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
    );
    // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
    // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
    // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
    // "0101010101010101010101010101010101010101010101010101010101010202"
    // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
    // the concatenation of these two hashes "b69..." and "965..."
    // notice that the order of these two matters
    }

    #[test]
    fn proof() {
    let input_data: Vec<H256> = gen_merkle_tree_data!();
    let merkle_tree = MerkleTree::new(&input_data);
    let proof = merkle_tree.proof(0);
    assert_eq!(proof,
    vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
    );
    // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
    // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn verifying() {
    let input_data: Vec<H256> = gen_merkle_tree_data!();
    let merkle_tree = MerkleTree::new(&input_data);
    let proof = merkle_tree.proof(0);
    assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }

    macro_rules! gen_merkle_tree_assignment2 {
        () => {{
            vec![
                (hex!("0000000000000000000000000000000000000000000000000000000000000011")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000022")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000033")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000044")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000055")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000066")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000077")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000088")).into(),
            ]
        }};
    }

    macro_rules! gen_merkle_tree_assignment2_another {
        () => {{
            vec![
                (hex!("1000000000000000000000000000000000000000000000000000000000000088")).into(),
                (hex!("2000000000000000000000000000000000000000000000000000000000000077")).into(),
                (hex!("3000000000000000000000000000000000000000000000000000000000000066")).into(),
                (hex!("4000000000000000000000000000000000000000000000000000000000000055")).into(),
                (hex!("5000000000000000000000000000000000000000000000000000000000000044")).into(),
                (hex!("6000000000000000000000000000000000000000000000000000000000000033")).into(),
                (hex!("7000000000000000000000000000000000000000000000000000000000000022")).into(),
                (hex!("8000000000000000000000000000000000000000000000000000000000000011")).into(),
            ]
        }};
    }

    #[test]
    fn assignment2_merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_assignment2!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6e18c8441bc8b0d1f0d4dc442c0d82ff2b4f38e2d7ca487c92e6db435d820a10")).into()
        );
    }

    #[test]
    fn assignment2_merkle_verify() {
        let input_data: Vec<H256> = gen_merkle_tree_assignment2!();
        let merkle_tree = MerkleTree::new(&input_data);
        for i in 0.. input_data.len() {
            let proof = merkle_tree.proof(i);
            assert!(verify(&merkle_tree.root(), &input_data[i].hash(), &proof, i, input_data.len()));
        }
        let input_data_2: Vec<H256> = gen_merkle_tree_assignment2_another!();
        let merkle_tree_2 = MerkleTree::new(&input_data_2);
        assert!(!verify(&merkle_tree.root(), &input_data[0].hash(), &merkle_tree_2.proof(0), 0, input_data.len()));
    }

    #[test]
    fn assignment2_merkle_proof() {
        use std::collections::HashSet;
        let input_data: Vec<H256> = gen_merkle_tree_assignment2!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(5);
        let proof: HashSet<H256> = proof.into_iter().collect();
        let p: H256 = (hex!("c8c37c89fcc6ee7f5e8237d2b7ed8c17640c154f8d7751c774719b2b82040c76")).into();
        assert!(proof.contains(&p));
        let p: H256 = (hex!("bada70a695501195fb5ad950a5a41c02c0f9c449a918937267710a0425151b77")).into();
        assert!(proof.contains(&p));
        let p: H256 = (hex!("1e28fb71415f259bd4b0b3b98d67a1240b4f3bed5923aa222c5fdbd97c8fb002")).into();
        assert!(proof.contains(&p));
    }
}
