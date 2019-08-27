use std::{convert::TryFrom, fs, sync::Arc};

use chain_client::{ChainClient, RpcChainClient};
use crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature};
use crypto::test_utils::KeyPair;
use failure::*;
use futures::{future, Future, Stream};
use grpcio::EnvBuilder;
use network::net::{build_network_service, NetworkService,Message};
use node::client;
use node_internal::node::Node;
use node_service::setup_node_service;
use sg_config::config::{load_from, NetworkConfig, NodeConfig, RpcConfig, WalletConfig};
use sgwallet::wallet::*;
use structopt::StructOpt;
use tokio::runtime::{Runtime, TaskExecutor};
use types::account_address::AccountAddress;
use futures_01::sync::mpsc::{UnboundedReceiver,UnboundedSender};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "stargate",
    author = "star-team",
    about = "stargate local node "
)]
struct Args {
    #[structopt(short = "l", long = "enable_logging")]
    pub enable_logging: bool,
    #[structopt(short = "s", long = "start_client")]
    pub start_client: bool,
    #[structopt(short = "c", long = "config_dir", default_value = "wallet")]
    pub config_dir: String,
    #[structopt(short = "f", long = "faucet_key_path", default_value = "wallet/key")]
    pub faucet_key_path: String,
}

pub struct Swarm {
    pub config: NodeConfig,
    tee_logs: bool,
}

fn launch_swarm(args: &Args) -> Result<Swarm> {
    let node_config = load_from(&(args.config_dir.to_string() + "/node.toml"))?;
    Ok(Swarm {
        config: node_config,
        tee_logs: true,
    })
}

fn load_from_file(faucet_account_file: &str) -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    match fs::read(faucet_account_file) {
        Ok(data) => {
            let private_key = Ed25519PrivateKey::try_from(&data[0..32]).unwrap();
            let public_key = Ed25519PublicKey::try_from(&data[32..]).unwrap();
            let keypair = KeyPair {
                private_key,
                public_key,
            };
            keypair
        }
        Err(e) => {
            panic!(
                "Unable to read faucet account file: {}, {}",
                faucet_account_file, e
            );
        }
    }
}

fn gen_node(
    executor: TaskExecutor,
    keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    wallet_config: &WalletConfig,
    network_service: NetworkService,sender:UnboundedSender<Message>,receiver:UnboundedReceiver<Message>
) -> (Node<RpcChainClient>) {
    let account_address = AccountAddress::from_public_key(&keypair.public_key);
    let env_builder_arc = Arc::new(EnvBuilder::new().build());
    let client = RpcChainClient::new(
        &wallet_config.chain_address,
        wallet_config.chain_port as u32,
    );

    let mut wallet =
        Wallet::new_with_client(account_address, keypair.clone(), Arc::new(client)).unwrap();

    Node::new(executor.clone(), wallet, keypair.clone(), network_service,sender,receiver)
}

fn main() {
    let args = Args::from_args();
    let swarm = launch_swarm(&args).unwrap();

    let mut rt = Runtime::new().unwrap();
    let executor = rt.executor();

    let keypair = load_from_file(&args.faucet_key_path);
    let (network_service, tx, rx) =
        build_network_service(&swarm.config.net_config, keypair.clone());

    let mut node = gen_node(executor, keypair, &swarm.config.wallet, network_service,tx,rx);
    //node.start_server(swarm.config.node_net_work.addr.parse().unwrap());

    let mut node_server = setup_node_service(&swarm.config, Arc::new(node));
    node_server.start();

    if args.start_client {
        let client = client::InteractiveClient::new_with_inherit_io(
            swarm.config.rpc_config.port, //Path::new(&faucet_key_file_path),
        );
        println!("Loading client...");
        let _output = client.output().expect("Failed to wait on child");
        println!("Exit client.");
    } else {
        let (tx, rx) = std::sync::mpsc::channel();
        ctrlc::set_handler(move || {
            tx.send(())
                .expect("failed to send unit when handling CTRL-C");
        })
        .expect("failed to set CTRL-C handler");
        println!("CTRL-C to exit.");
        rx.recv()
            .expect("failed to receive unit when handling CTRL-C");
    }
}
