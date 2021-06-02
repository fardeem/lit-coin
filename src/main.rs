mod block;
mod blockchain;

use secp256k1::{Secp256k1, Message, All};
use secp256k1::rand::rngs::OsRng;
use secp256k1::bitcoin_hashes::sha256 as sec_sha256;

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
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::{io::AsyncBufReadExt, sync::mpsc};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

static KEYS: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());
static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
static TX_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("transactions"));
static BLOCKCHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("blockchain"));

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: usize,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
enum ListMode {
    ALL,
    One(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct ListRequest {
    mode: ListMode,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResponse {
    mode: ListMode,
    data: Transaction,
    receiver: String,
}

enum EventType {
    Response(ListResponse),
    Input(String),
}

#[derive(NetworkBehaviour)]
struct BlockchainNetworkBehavior {
    floodsub: Floodsub,
    mdns: TokioMdns,
    #[behaviour(ignore)]
    _response_sender: mpsc::UnboundedSender<ListResponse>,
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for BlockchainNetworkBehavior {
    fn inject_event(&mut self, event: FloodsubEvent) {
        match event {
            FloodsubEvent::Message(msg) => {
                print!("{}", str::from_utf8(&msg.data).unwrap());
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
    let secp = Secp256k1::new();
    let mut rng = OsRng::new().expect("OsRng");
    let (secret_key, public_key) = secp.generate_keypair(&mut rng);

    let secp2 = Secp256k1::new();
    let mut rng2 = OsRng::new().expect("OsRng");
    let (secret_key2, public_key2) = secp.generate_keypair(&mut rng2);

    let mut bc = blockchain::Blockchain::new();
    let reward = block::Reward {
        address: "".to_owned(), 
        amount: 10
    };

    let reward2 = block::Reward {
        address: "".to_owned(), 
        amount: 10
    };
    let mut block = block::Block::new(0, "".to_owned(), "timestamp".to_owned(), block::Transaction::new("".to_owned(), "".to_owned(), "".to_owned(), 0, secret_key, public_key, secp), "".to_owned(), 0, reward);
    let mut block2 = block::Block::new(0, block.hash.to_string(), "timestamp".to_owned(), block::Transaction::new("".to_owned(), "".to_owned(), "".to_owned(), 0, secret_key2, public_key2, secp2), "".to_owned(), 0, reward2);
    block.hash = block::calculate_hash(&block);
    block2.hash = block::calculate_hash(&block2);
    bc.add_block(block);
    bc.add_block(block2);

    

    pretty_env_logger::init();
    info!("Peer Id: {}", PEER_ID.clone());

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
        _response_sender: response_sender,
    };

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
                    info!("Unhandled Swarm Event: {:?}", event);
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
                    "list blocks" => handle_list_blocks().await,
                    "print balance" => handle_print_balance().await,
                    cmd if cmd.starts_with("tx ") => handle_new_transaction(cmd).await,
                    cmd => error!("unknown command - {}", cmd),
                },
            }
        }
    }
}

async fn handle_list_peers(swarm: &mut Swarm<BlockchainNetworkBehavior>) {
    info!("Discovered Peers:");
    let nodes = swarm.mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }
    unique_peers.iter().for_each(|p| info!("{}", p));
}

async fn handle_print_balance() {
    info!("Your balance is: ")
}

async fn handle_list_blocks() {
    info!("Listing Blocks......");

    info!("Block 1");
    info!("Block 2");
}

async fn create_new_transaction(to: &str, amount: &str) -> Result<()> {
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let _tx = Transaction {
        from: KEYS.public().into_peer_id().to_string(),
        to: to.to_string(),
        amount: amount.parse::<usize>().unwrap(),
        timestamp: since_the_epoch.as_secs(),
    };

    Ok(())
}

async fn handle_new_transaction(cmd: &str) {
    if let Some(rest) = cmd.strip_prefix("tx") {
        // Transactions are formatted at to|amount

        let elements: Vec<&str> = rest.split("|").collect();

        if elements.len() < 2 {
            info!("too few arguments - Format: to|amount");
        } else {
            let to_address = elements.get(0).expect("to address is there");
            let amount = elements.get(1).expect("amount is there");

            if let Err(e) = create_new_transaction(to_address, amount).await {
                error!("error creating recipe: {}", e);
            };
        }
    }
}
