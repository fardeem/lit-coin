use super::block::{calculate_hash, Block};
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
        println!("A new block added {}, previous block is {}", block.hash, block.previous_hash)
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
            println!("debug {:?}", block.timestamp);
            let temp = block.timestamp.parse::<usize>().unwrap();
            if val.is_empty() && temp > latest{
                latest = temp;
                res = Some(block);
            }
        }
        res
    }
}
