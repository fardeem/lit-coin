use super::block::{calculate_hash, Block, Reward, Transaction};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Blockchain {
    // lock: Arc<Mutex<usize>>,
    pub map: HashMap<String, Vec<Block>>,
    pub block_table: HashMap<String, Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            // lock: Arc::new(Mutex::new(0)),
            map: HashMap::new(),
            block_table: HashMap::new(),
        }
    }

    pub fn get_previous(&self, block: &Block) -> Option<&Block> {
        self.block_table.get(&block.previous_hash)
    }

    pub fn add_block(&mut self, block: Block) {
        if !self.validate_block(&block) {
            println!("INVALID BLOCK");
            return;
        }

        if self.map.contains_key(&block.hash) {
            return 
        }

        self.remove_block(&block);
        self.block_table
            .insert(block.hash.to_string(), block.clone());
        self.map.insert(block.hash.to_string(), vec![]);
        self.try_insert(block.clone());
        // println!("A new block added {}, previous block is {}", block.hash, block.previous_hash);
        // println!("balance is {:?}", self.get_balances());
        // self.print_blockchain();
    }

    pub fn try_insert(&mut self, block: Block) {
        if !block.previous_hash.is_empty() {
            self.map
                .entry(block.previous_hash.to_string())
                .or_insert_with(Vec::new)
                .push(block);
        }
    }

    pub fn remove_block(&mut self, block: &Block) {
        let mut curr = block;
        let mut prev = block;
        for _ in 0..3 {
            curr = prev;
            if let Some(previous) = self.get_previous(curr) {
                prev = previous;
            } else {
                return;
            }
        }

        self.map.insert(prev.hash.to_string(), vec![curr.clone()]);
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        match self.get_previous(&block) {
            None => calculate_hash(&block) == block.hash,
            Some(previous_block) => {
                previous_block.index + 1 == block.index
                    && previous_block.hash == block.previous_hash
                    && calculate_hash(&block) == block.hash
            }
        }
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        let mut res: Option<&Block> = None;
        let mut latest = 0;
        for (key, val) in self.map.iter() {
            let block = self.block_table.get(key).unwrap();
            let temp = block.timestamp.parse::<usize>().unwrap();
            if val.is_empty() && temp > latest{
                latest = temp;
                res = Some(block);
            }
        }
        res
    }

    pub fn print_blockchain(&self) {
        for (k, _v) in self.map.iter() {
            println!("Block: {}, previous block: {}", k, self.block_table[k].previous_hash)
        }
    }

    pub fn get_balances(&self) -> HashMap<String, i64> { // TODO
        // get latest block todo 
        let mut block = self.get_latest_block().unwrap();
        let mut balances: HashMap<String, i64> = HashMap::new();
        while block.previous_hash != "" {
            //subtract from
            let from_balance = balances.entry(block.tx.from.to_string()).or_insert(0);
            *from_balance -= block.tx.amount as i64;
            //add to
            let to_balance = balances.entry(block.tx.to.to_string()).or_insert(0);
            *to_balance += block.tx.amount as i64;
            //add reward
            let reward_balance = balances.entry(block.reward.address.to_string()).or_insert(0);
            *reward_balance += block.reward.amount as i64;
            //go to next block
            block = &self.block_table[&block.previous_hash];
        }
        balances
    }
}
