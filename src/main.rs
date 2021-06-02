// #![feature(proc_macro_hygiene, decl_macro)]

// // #[macro_use] extern crate rocket;
// use async_std::net::TcpListener;
// use async_std::net::TcpStream;
// use async_std::prelude::*;
// use futures::stream::StreamExt;
// use sha256::digest;
// use std::time::{SystemTime, UNIX_EPOCH};

mod block;
mod blockchain;

fn main() {
  block::generate_new_block(block::Transaction::new("".to_owned(), "".to_owned(), "".to_owned(), 0, "".to_owned(), "".to_owned()));
}

// #[get("/blocks")]
// fn blocks() -> Vec<Blocks> {
//   blockchain
// }

// #[get("/peers")]
// fn peers() -> Vec<Blocks> {
//   for socket in sockets {

//   }
// }

// #[post("/mineBlock", data = "<block_data>")]
// fn mine_block(block_data: String) {

// }

// //should peer be an IP address type ? 
// #[post("/addPeer", data = "<peer>")]
// fn add_peer(peer: String) {

// }

// fn main() {
//   let blockchain = [getGenesisBlock()];
//   let sockets = vec![];
// }

// #[async_std::main]
// async fn init_p2p_server() {
//   let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

//     listener
//         .incoming()
//         .for_each_concurrent(None, |tcpstream| async move {
//             let tcpstream = tcpstream.unwrap();
//             init_connection(tcpstream).await;
//         })
//         .await;
// }

// fn generate_new_block(data: String) -> Block {
//   let previous_block = get_latest_block();
//   let index = previous_block.index + 1;
//   let start = SystemTime::now();
//   let timestamp = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
//   let hash = calculateHash(nextIndex, previousBlock.hash, nextTimestamp, blockData);
  
//   Block {
//     index,
//     previous_hash: previous_block.hash,
//     timestamp,
//     data,
//     hash
//   }
// }

// pub fn get_genesis_block() -> Block {
// Block {
//   index: 0,
//   previous_hash: "0",
//   timestamp: 1465154705,
//   data: "litcoin genesis block",
//   timestamp: "816534932c2b7154836da6afc367695e6337db8a921823784c14378abed4f7d7"     
// }
// }

// fn calculate_block_hash(block: Block) -> String {
// calculate_hash(block.index, block.previous_hash, block.timestamp, block.data)
// }
// pub fn is_valid_block(new_block: Block, previous_block: Block) -> bool {
// if previous_block.index + 1 != new_block.index || previous_block.hash != new_block.previous_hash || calculate_block_hash(new_block) != new_block.hash {
//   return false;
// }
// true
// }

// async fn init_connection(stream: &mut TcpStream) {
//   sockets.append(stream.clone());
//   // ask peer for latest block that it has
//   stream.clone().write
//   // define how each incoming message should be responded to as mentioned in the JS version
//   // put the stream.read in a loop so that connection does not drop ? 
// }

// fn replace_chain(new_blocks: Vec<Block>) {
//   if is_valid_chain(new_blocks) && new_blocks.length > blockchain.length {
//       print!("Replacing existing blockchain with new blockchain");
//       blockchain = new_blocks;
//       broadcast(responseLatestMsg());
//   } else {
//       print!("Received blockchain invalid");
//   }
// }
