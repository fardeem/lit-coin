use sha256::digest;
use std::time::{SystemTime};
use chrono::offset::Utc;
use chrono::DateTime;

const DIFFICULTY: &str = "0000";
const REWARD_AMOUNT: usize = 90;

pub struct Block {
    index: usize,
    previous_hash: String,
    timestamp: String,
    tx: Transaction,
    hash: String, 
    nonce: usize,
    reward: Reward
}

impl Block {
    pub fn new(index: usize, previous_hash: String, timestamp: String, tx: Transaction, hash: String, nonce: usize, reward: Reward) -> Self {
        Self {
            index,
            previous_hash,
            timestamp,
            tx,
            hash,
            nonce,
            reward,
        }
    }
}

//   Placeholder
pub struct Transaction {
    from: String,
    to: String,
    timestamp: String,
    amount: usize,
    signature: String,
    pk: String,
}

impl Transaction {
    pub fn new(from: String, to: String, timestamp: String, amount: usize, signature: String, pk: String) -> Self {
        Self {
            from, 
            to, 
            timestamp,
            amount, 
            signature, 
            pk
        }
    }

    pub fn serialize(&self) -> String {
        format!("{}{}{}{}{}{}",self.from, self.to, self.timestamp, self.amount, self.signature, self.pk)
    }
}

pub struct Reward {
    // id, address? it'll be the same address used for the p2p conenction
    address: String,
    amount: usize,
}

fn get_latest_block() -> Block {
    //   PlaceHolder
    let reward = Reward {
        address: "".to_owned(), 
        amount: REWARD_AMOUNT
    };
    Block::new(0, "".to_owned(), "timestamp".to_owned(), Transaction::new("".to_owned(), "".to_owned(), "".to_owned(), 0, "".to_owned(), "".to_owned()), "".to_owned(), 0, reward)
}

pub fn generate_new_block(transaction: Transaction) -> Block {
    let previous_block = get_latest_block();
    let index = previous_block.index + 1;
    let time = SystemTime::now();
    let timestamp: DateTime<Utc> = time.into();
    let reward = Reward {
        address: "".to_owned(), 
        amount: REWARD_AMOUNT
    };
    let mut block = Block::new(index, previous_block.hash, timestamp.to_string(), transaction, "".to_owned(), 0, reward);

    let mut i = 0;
    loop {
        block.nonce = i;
        let hash = calculate_hash(&block);
        if is_hash_valid(&hash) {
            println!("{} Found correct hash", hash);
            block.hash = hash;
            block.reward = Reward {
                address: "".to_owned(), 
                amount: REWARD_AMOUNT
            };
            return block;
        } else {
            println!("{} Incorrect hash", hash);
            i += 1;
        }
    }
}

fn calculate_hash(block: &Block) -> String {
    let hash = format!("{}{}{}{}{}", block.index, block.nonce, block.previous_hash, block.timestamp, block.tx.serialize());
    digest(hash)
}

fn is_hash_valid(hash: &str) -> bool {
    hash.starts_with(DIFFICULTY)
}
