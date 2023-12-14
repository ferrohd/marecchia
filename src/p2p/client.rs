use futures::Stream;
use libp2p::{
    futures::{
        channel::{mpsc, oneshot},
        SinkExt,
    },
    identity::{self, PeerId},
    multiaddr::Multiaddr,
    request_response::ResponseChannel,
    SwarmBuilder,
};
use libp2p_webrtc_websys as webrtc_websys;
use std::{collections::HashSet, error::Error, time::Duration};

use super::{
    behaviour::{ComposedSwarmBehaviour, SegmentResponse},
    event_loop::{EventLoop, LoopCommand, LoopEvent},
};

pub async fn new(
    secret_key_seed: Option<u8>,
) -> Result<(Client, impl Stream<Item = LoopEvent>), Box<dyn Error>> {
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
        .with_other_transport(|key| {
            webrtc_websys::Transport::new(webrtc_websys::Config::new(&key))
        })?
        .with_behaviour(|key| ComposedSwarmBehaviour::from(key))?
        .with_swarm_config(|c| {
            c.with_max_negotiating_inbound_streams(32)
                .with_idle_connection_timeout(Duration::from_secs(0))
                .with_dial_concurrency_factor(5.try_into().unwrap())
        })
        .build();

    let (command_sender, command_receiver) = mpsc::channel(20);
    let (event_sender, event_receiver) = mpsc::channel(20);

    wasm_bindgen_futures::spawn_local(async move {
        EventLoop::new(swarm, command_receiver, event_sender)
            .run()
            .await;
    });

    Ok((Client(command_sender), event_receiver))
}

#[derive(Clone)]
pub struct Client(mpsc::Sender<LoopCommand>);

impl Client {
    /// Dial the given peer at the given address.
    pub async fn dial(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(LoopCommand::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Advertise the local node as the provider of the given file on the DHT.
    pub async fn start_providing(&mut self, segment_id: String) {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(LoopCommand::StartProviding { segment_id, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.");
    }

    /// Find the providers for the given file on the DHT.
    pub async fn get_providers(&mut self, segment_id: String) -> HashSet<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(LoopCommand::GetProviders { segment_id, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Request the content of the given file from the given peer.
    pub async fn request_segment(
        &mut self,
        peer: PeerId,
        segment_id: String,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(LoopCommand::RequestSegment {
                segment_id,
                peer,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Respond with the provided file content to the given request.
    pub async fn respond_segment(
        &mut self,
        file: Option<Vec<u8>>,
        channel: ResponseChannel<SegmentResponse>,
    ) {
        self.0
            .send(LoopCommand::RespondSegment {
                segment_data: file,
                channel,
            })
            .await
            .expect("Command receiver not to be dropped.");
    }
}
