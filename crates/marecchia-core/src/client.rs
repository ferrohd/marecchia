use js_sys::Uint8Array;
use libp2p::{
    PeerId, SwarmBuilder, Transport,
    core::upgrade::Version,
    futures::{
        SinkExt,
        channel::{mpsc, oneshot},
    },
    identity,
    multiaddr::{Multiaddr, Protocol},
    noise,
    rendezvous::Namespace,
    websocket_websys, yamux,
};
use libp2p_webrtc_websys as webrtc_websys;
use std::{num::NonZeroU8, panic, time::Duration};
use tracing_subscriber::{fmt::format::Pretty, prelude::*};
use tracing_web::{MakeWebConsoleWriter, performance_layer};
use wasm_bindgen::prelude::*;

use super::{
    behaviour::ComposedSwarmBehaviour,
    event_loop::{Command, EventLoop},
};

#[wasm_bindgen]
pub fn new_p2p_client(stream_namespace: String) -> Result<P2PClient, JsError> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .without_time() // std::time is not available in browsers, see note below
        .with_writer(MakeWebConsoleWriter::new()); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init(); // Install these as subscribers to tracing events

    let namespace = Namespace::new(stream_namespace)?;
    tracing::info!("Starting P2P client with stream namespace: {:?}", namespace);

    // Create a public/private key pair, either random or based on a seed.
    let keypair = identity::Keypair::generate_ed25519();
    tracing::debug!("Peer ID: {:?}", keypair.public().to_peer_id());

    // Build the Swarm, connecting the lower layer transport logic with the
    // higher layer network behaviour logic.
    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_wasm_bindgen()
        .with_other_transport(|key| {
            websocket_websys::Transport::default()
                .upgrade(Version::V1)
                .authenticate(noise::Config::new(key).unwrap())
                .multiplex(yamux::Config::default())
                .boxed()
        })?
        .with_other_transport(|key| webrtc_websys::Transport::new(webrtc_websys::Config::new(key)))?
        .with_relay_client(|key: &_| noise::Config::new(key), yamux::Config::default)?
        // TODO: implement bandwidth metrics
        //.with_bandwidth_metrics(...)
        .with_behaviour(ComposedSwarmBehaviour::new)?
        .with_swarm_config(|c| {
            c.with_max_negotiating_inbound_streams(16)
                .with_idle_connection_timeout(Duration::from_secs(60))
                .with_dial_concurrency_factor(NonZeroU8::new(5).unwrap())
        })
        .build();

    // Listen for inbound connections
    //let addr = Multiaddr::empty().with(Protocol::WebRTCDirect);
    //swarm.listen_on(addr).map_err(|e| ClientError::ListenError);
    //tracing::info!("Listening on {:?}", addr);

    let (command_send, command_recv) = mpsc::channel(20);

    tracing::info!("P2P client started");

    let rendezvous_id = PeerId::random();
    let rendezvous_addr = Multiaddr::empty()
        .with(Protocol::Dns("rendezvous.marecchia.io".into()))
        .with(Protocol::P2p(rendezvous_id));

    tracing::info!("Dialing rendezvous server at {:?}", rendezvous_addr);
    swarm.dial(rendezvous_addr)?;

    wasm_bindgen_futures::spawn_local(async move {
        EventLoop::new(namespace, swarm, command_recv).run().await;
    });

    Ok(P2PClient(command_send))
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct P2PClient(mpsc::Sender<Command>);

#[wasm_bindgen]
impl P2PClient {
    /// Advertise the local node as the provider of the given file on the DHT.
    pub async fn send_segment(
        &mut self,
        segment_id: String,
        segment: Uint8Array,
    ) -> Result<(), JsError> {
        let data = segment.to_vec();
        self.0
            .send(Command::ProvideSegment { segment_id, data })
            .await?;
        Ok(())
    }

    /// Request the content of the given file from the given peer.
    pub async fn request_segment(&mut self, segment_id: String) -> Result<Uint8Array, JsError> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(Command::RequestSegment { segment_id, sender })
            .await?;

        let segment = receiver.await??;
        let buf = Uint8Array::from(segment.as_slice());

        Ok(buf)
    }

    pub async fn quit(&mut self) -> Result<(), JsError> {
        self.0.send(Command::Quit).await?;
        Ok(())
    }
}
