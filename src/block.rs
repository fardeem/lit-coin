use sha256::digest;
use std::time::{SystemTime};
use chrono::offset::Utc;
use chrono::DateTime;
use secp256k1::{Secp256k1, Message, All};
use secp256k1::rand::rngs::OsRng;
use secp256k1::bitcoin_hashes::sha256 as sec_sha256;

const DIFFICULTY: &str = "0000";
const REWARD_AMOUNT: usize = 90;

pub struct Block {
    pub index: usize,
    pub previous_hash: String,
    timestamp: String,
    tx: Transaction,
    pub hash: String, 
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
    signature: secp256k1::Signature,
    pk: secp256k1::PublicKey,
}

impl Transaction {
    pub fn new(from: String, to: String, timestamp: String, amount: usize, sk: secp256k1::SecretKey, pk: secp256k1::PublicKey, secp: Secp256k1<All>) -> Self {
        let message = Message::from_hashed_data::<sec_sha256::Hash>(format!("{}{}{}{}",from, to, timestamp, amount).as_bytes());
        Self {
            from, 
            to, 
            timestamp,
            amount, 
            signature: secp.sign(&message, &sk), 
            pk
        }
    }

    pub fn verify_signature(&self, secp: Secp256k1<All>) -> bool {
        let message = Message::from_hashed_data::<sec_sha256::Hash>(format!("{}{}{}{}",self.from, self.to, self.timestamp, self.amount).as_bytes());
        secp.verify(&message, &self.signature, &self.pk).is_ok()
    }

    pub fn serialize(&self) -> String {
        format!("{}{}{}{}{}{}",self.from, self.to, self.timestamp, self.amount, self.signature.to_string(), self.pk.to_string())
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
    //REMOVE LATER
    let secp = Secp256k1::new();
    let mut rng = OsRng::new().expect("OsRng");
    let (secret_key, public_key) = secp.generate_keypair(&mut rng);

    Block::new(0, "".to_owned(), "timestamp".to_owned(), Transaction::new("".to_owned(), "".to_owned(), "".to_owned(), 0, secret_key, public_key, secp), "".to_owned(), 0, reward)
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

pub fn calculate_hash(block: &Block) -> String {
    let hash = format!("{}{}{}{}{}", block.index, block.nonce, block.previous_hash, block.timestamp, block.tx.serialize());
    digest(hash)
}

fn is_hash_valid(hash: &str) -> bool {
    hash.starts_with(DIFFICULTY)
}
