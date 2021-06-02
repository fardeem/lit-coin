use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::block::{Block, calculate_hash};

pub struct Blockchain {
    lock: Arc<Mutex<usize>>,
    map: HashMap<String, Vec<Block>> 
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            lock: Arc::new(Mutex::new(0)),
            map: HashMap::new() 
        }
    }

    pub fn add_block(&mut self, block: Block) {
        todo!()
    }

    pub fn validate_block(&mut self, block: Block, previous_block: Block) -> bool {
        previous_block.index + 1 == block.index && previous_block.hash == block.previous_hash && calculate_hash(&block) == block.hash
    }

    pub fn remove_block(&mut self, block: Block) {
        todo!()
    }
}