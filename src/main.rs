mod block;
mod blockchain;

use block::{generate_new_block, Block, Reward, Transaction};
use blockchain::Blockchain;

use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{MdnsEvent, TokioMdns},
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    NetworkBehaviour, PeerId, Transport,
};
use log::error;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::{io::AsyncBufReadExt, sync::mpsc};

static KEYS: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());
static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
static TX_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("transactions"));
static BLOCKCHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("blockchain"));

enum EventType {
    Response(Block),
    Input(String),
}

#[derive(NetworkBehaviour)]
struct BlockchainNetworkBehavior {
    floodsub: Floodsub,
    mdns: TokioMdns,
    #[behaviour(ignore)]
    response_sender: mpsc::UnboundedSender<Block>,
    #[behaviour(ignore)]
    chain: Blockchain,
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for BlockchainNetworkBehavior {
    fn inject_event(&mut self, event: FloodsubEvent) {
        match event {
            FloodsubEvent::Message(msg) => {
                if let Ok(tx) = serde_json::from_slice::<Transaction>(&msg.data) {
                    println!(
                        "INFO: Received transaction. \nContents..... \nFrom: {}, \nTo: {}, \nAmount: {}",
                        tx.from, tx.to, tx.amount
                    );

                    let block = generate_new_block(tx, self.chain.get_latest_block().unwrap());
                    self.chain.add_block(block.clone());

                    if let Err(e) = self.response_sender.send(block) {
                        println!("error sending response over channel, {}", e);
                    }
                } else if let Ok(block) = serde_json::from_slice::<Block>(&msg.data) {
                    println!(
                        "Recevied block with hash: {} and previous hash: {}",
                        block.hash, block.previous_hash
                    );
                    self.chain.add_block(block)
                }
            }
            _ => (),
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for BlockchainNetworkBehavior {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Some queue to handle async message back and forth
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();

    // We create a transport layer,
    // and use auth keys to secure it
    // We using our KEYS to sign it
    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&KEYS)
        .expect("can create auth keys");

    // Make the transport layer
    // Some galaxy brain p2p thing going on here, wont type out the details
    // but its cool and works
    let transp = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated()) // XX Handshake pattern, IX exists as well and IK - only XX currently provides interop with other libp2p impls
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let mut behaviour = BlockchainNetworkBehavior {
        floodsub: Floodsub::new(PEER_ID.clone()),
        mdns: TokioMdns::new().expect("can create mdns"),
        response_sender,
        chain: Blockchain::new(),
    };

    let reward = Reward {
        address: "".to_owned(),
        amount: 10,
    };

    let mut block = Block::new(
        0,
        "".to_string(),
        "1".to_owned(),
        Transaction::new("".to_owned(), "".to_owned(), "".to_owned(), 0),
        "".to_owned(),
        0,
        reward,
    );
    block.hash = block::calculate_hash(&block);
    // Create the genesis block
    behaviour.chain.add_block(block);

    behaviour.floodsub.subscribe(BLOCKCHAIN_TOPIC.clone());
    behaviour.floodsub.subscribe(TX_TOPIC.clone());

    let mut swarm = SwarmBuilder::new(transp, behaviour, PEER_ID.clone())
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("can get a local socket"),
    )
    .expect("swarm can be started");

    // End Libp2p setup

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    loop {
        // Capture all the inputs in stdin
        // events coming in from the swarm
        // and in the async queue
        // and handle them all together
        let evt = {
            tokio::select! {
                line = stdin.next_line() => Some(EventType::Input(line.expect("can get line").expect("can read line from stdin"))),
                event = swarm.next() => {
                    println!("Unhandled Swarm Event: {:?}", event);
                    None
                },
                response = response_rcv.recv() => Some(EventType::Response(response.expect("response exists"))),
            }
        };

        if let Some(event) = evt {
            match event {
                EventType::Response(resp) => {
                    let json = serde_json::to_string(&resp).expect("can jsonify response");
                    swarm
                        .floodsub
                        .publish(BLOCKCHAIN_TOPIC.clone(), json.as_bytes());
                }
                EventType::Input(line) => match line.as_str() {
                    "list peers" => handle_list_peers(&mut swarm).await,
                    cmd if cmd.starts_with("tx ") => handle_new_transaction(cmd, &mut swarm).await,
                    cmd => error!("unknown command - {}", cmd),
                },
            }
        }
    }
}

async fn handle_list_peers(swarm: &mut Swarm<BlockchainNetworkBehavior>) {
    println!("Discovered Peers:");
    let nodes = swarm.mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }
    unique_peers.iter().for_each(|p| println!("{}", p));
}

async fn handle_new_transaction(cmd: &str, swarm: &mut Swarm<BlockchainNetworkBehavior>) {
    if let Some(rest) = cmd.strip_prefix("tx") {
        // Transactions are formatted at to|amount

        let elements: Vec<&str> = rest.split("|").collect();

        if elements.len() < 2 {
            println!("too few arguments - Format: to|amount");
            return;
        }

        let to_address = elements.get(0).expect("to address is there");
        let amount = elements.get(1).expect("amount is there");

        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let tx = Transaction {
            from: KEYS.public().into_peer_id().to_string(),
            to: to_address.to_string(),
            amount: amount.parse::<usize>().unwrap(),
            timestamp: since_the_epoch.as_secs().to_string(),
        };

        swarm.floodsub.publish(
            TX_TOPIC.clone(),
            serde_json::to_string(&tx).unwrap().as_bytes(),
        )
    }
}
