use sha256::digest;
use std::time::{SystemTime, UNIX_EPOCH};

const difficulty: &str = "0000";

pub struct Block {
    index: usize,
    previous_hash: String,
    timestamp: String,
    tx: Transaction,
    hash: String, 
    nonce: usize,
    reward: usize
}

  
impl Block {
    pub fn new(index: usize, previous_hash: String, timestamp: String, tx: Transaction, hash: String, nonce: usize, reward: usize) -> Self {
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
    amount: usize,
    signature: String,
    pk: String,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: usize, signature: String, pk: String) -> Self {
        Self {
        from, to, amount, signature, pk
        }
    }
}

fn get_latest_block() -> Block {
    //   PlaceHolder
    Block::new(0, "".to_owned(), "timestamp".to_owned(), Transaction::new("".to_owned(), "".to_owned(), 0, "".to_owned(), "".to_owned()), "".to_owned(), 0, 0)
}

fn is_hash_valid(hash: &str) -> bool {
    hash.starts_with(difficulty)
}

pub fn generate_new_block(transaction: Transaction) -> Block {
    let previous_block = get_latest_block();
    let index = previous_block.index + 1;
    let start = SystemTime::now();
    let timestamp = start.duration_since(UNIX_EPOCH).expect("Time went backwards").to_string();
    let mut block = Block::new(index, previous_block.hash, timestamp, transaction, "".to_owned(), 0);

    let mut i = 0;
    loop {
        block.nonce = i;
        let hash = calculate_hash(block);
        if is_hash_valid(&hash) {
            println!("{} Found correct hash", hash);
            block.hash = hash;
            return block;
        } else {
            println!("{} Incorrect hash", hash);
            i += 1;
        }
    }
}


fn calculate_hash(block: Block) -> String {
    let hash = format!("{}{}{}{}{}", block.index.to_string(), block.nonce.to_string(), block.previous_hash, block.timestamp, block.tx);
    digest(hash)
}
