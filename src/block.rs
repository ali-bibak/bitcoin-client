extern crate rand;

use serde::{Serialize, Deserialize};
use ring::digest::{SHA256, digest};
use std::time::{SystemTime};
use rand::Rng;

use crate::crypto::hash::{H256, Hashable};
use crate::transaction::{Transaction};

/// A block in the blockchain
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: Header,
    content: Content,
}

impl Block {
    pub fn new(parent: H256, difficulty: H256, transactions: Vec<Transaction>, merkle_root: H256) -> Self {
        let mut rng = rand::thread_rng();
        let nonce: u32 = rng.gen();
        let timestamp = SystemTime::now();
        let block: Block = Block {
            header: Header {
                parent,
                nonce,
                difficulty,
                timestamp,
            },
            content: Content {
                transactions,
                merkle_root,
            },
        };
        return block;
    }

    pub fn get_parent(&self) -> H256 {
        return self.header.parent;
    }

    pub fn get_difficulty(&self) -> H256 {
        return self.header.difficulty;
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        return self.header.hash();
    }
}

/// The header of a block
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    parent: H256,
    nonce: u32,
    difficulty: H256,
    timestamp: SystemTime,
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        let serialized = bincode::serialize(&self).unwrap();
        let hashed = digest(&SHA256, &serialized);
        let hashed256 = H256::from(hashed);
        return hashed256;
    }
}

/// The content of a block
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    transactions: Vec<Transaction>,
    merkle_root: H256,
}

#[cfg(any(test, test_utilities))]
pub mod test {
    use super::*;
    use crate::crypto::hash::H256;
    use crate::crypto::merkle::{MerkleTree};
    use crate::blockchain::Blockchain;

    pub fn generate_random_block(parent: &H256) -> Block {
        let difficulty: H256 = Blockchain::get_difficulty().into();
        let mut transactions: Vec<Transaction> = Vec::new();
        let transaction = Transaction::new("rand in".to_string(), "rand_out".to_string());
        transactions.push(transaction);
        let merkle_tree = MerkleTree::new(&transactions);
        let merkle_root = merkle_tree.root();

        let block: Block = Block::new(parent.clone(), difficulty, transactions, merkle_root);
        return block;
    }
}
