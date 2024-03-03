use libp2p::{
    futures::{
        channel::{mpsc, oneshot},
        SinkExt,
    },
    identity::{self, PeerId},
    multiaddr::Multiaddr,
    rendezvous::Namespace,
    SwarmBuilder,
};
use libp2p_webrtc_websys as webrtc_websys;
use std::{error::Error, num::NonZeroU8, time::Duration};

use super::{
    behaviour::ComposedSwarmBehaviour,
    event_loop::{Command, EventLoop},
};

pub async fn new(stream_id: String, secret_key_seed: Option<u8>) -> Result<Client, Box<dyn Error>> {
    let namespace = Namespace::new(stream_id)?;

    // Create a public/private key pair, either random or based on a seed.
    let keypair = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(&mut bytes).expect(
                "this returns `Err` only if the length is wrong; the length is correct; qed",
            )
        }
        None => identity::Keypair::generate_ed25519(),
    };

    // Build the Swarm, connecting the lower layer transport logic with the
    // higher layer network behaviour logic.
    let swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_wasm_bindgen()
        .with_other_transport(|key| webrtc_websys::Transport::new(webrtc_websys::Config::new(key)))?
        .with_behaviour(|key| ComposedSwarmBehaviour::from(key))?
        .with_swarm_config(|c| {
            c.with_max_negotiating_inbound_streams(32)
                .with_idle_connection_timeout(Duration::from_secs(0))
                .with_dial_concurrency_factor(NonZeroU8::new(5).unwrap())
        })
        .build();

    let (command_send, command_recv) = mpsc::channel(20);

    wasm_bindgen_futures::spawn_local(async move {
        EventLoop::new(namespace, swarm, command_recv).run().await;
    });

    Ok(Client(command_send))
}

#[derive(Clone)]
pub struct Client(mpsc::Sender<Command>);

impl Client {
    /// Dial the given peer at the given address.
    pub async fn dial(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(Command::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Advertise the local node as the provider of the given file on the DHT.
    pub async fn start_providing(&mut self, segment_id: String, data: Vec<u8>) {
        self.0
            .send(Command::ProvideSegment { segment_id, data })
            .await
            .expect("Command receiver not to be dropped.");
    }

    /// Request the content of the given file from the given peer.
    pub async fn request_segment(
        &mut self,
        segment_id: String,
    ) -> Result<Vec<u8>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(Command::RequestSegment { segment_id, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
}
