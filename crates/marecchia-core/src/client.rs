use futures::{
    channel::{mpsc::SendError, oneshot::Canceled},
    StreamExt,
};
use js_sys::Uint8Array;
use libp2p::{
    futures::{
        channel::{mpsc, oneshot},
        SinkExt,
    },
    identity,
    multiaddr::{Multiaddr, Protocol},
    rendezvous::{Namespace, NamespaceTooLong},
    swarm::DialError,
    PeerId, SwarmBuilder,
};
use libp2p_webrtc_websys as webrtc_websys;
use std::{num::NonZeroU8, panic, str::FromStr, time::Duration};
use tracing_subscriber::{fmt::format::Pretty, prelude::*};
use tracing_web::{performance_layer, MakeWebConsoleWriter};
use wasm_bindgen::prelude::*;

use super::{
    behaviour::ComposedSwarmBehaviour,
    event_loop::{Command, EventLoop, RequestError},
};

#[wasm_bindgen]
pub fn new_p2p_client(stream_namespace: String) -> Result<P2PClient, ClientError> {
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
        .with_other_transport(|key| webrtc_websys::Transport::new(webrtc_websys::Config::new(key)))
        .map_err(|_| ClientError::ConfigError)?
        .with_behaviour(|key| ComposedSwarmBehaviour::from(key))
        .map_err(|_| ClientError::ConfigError)?
        .with_swarm_config(|c| {
            c.with_max_negotiating_inbound_streams(32)
                .with_idle_connection_timeout(Duration::from_secs(60))
                .with_dial_concurrency_factor(NonZeroU8::new(5).unwrap())
        })
        .build();

    // Listen for inbound connections
    //let addr = Multiaddr::empty().with(Protocol::WebRTCDirect);
    //swarm.listen_on(addr).map_err(|e| ClientError::ListenError);
    //tracing::info!("Listening on {:?}", addr);

    let (command_send, command_recv) = mpsc::channel(20);

    wasm_bindgen_futures::spawn_local(async move {
        EventLoop::new(namespace, swarm, command_recv).run().await;
    });

    tracing::info!("P2P client started");

    let rendezvous_id = PeerId::random();
    let rendezvous_addr = Multiaddr::empty()
        .with(Protocol::Dns("rendezvous.marecchia.io".into()))
        .with(Protocol::P2p(rendezvous_id));

    swarm
        .dial(rendezvous_addr)
        .map_err(|_| ClientError::DialError)?;

    tracing::info!("Dialing rendezvous server at {:?}", rendezvous_addr);

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
    ) -> Result<(), ClientError> {
        let data = segment.to_vec();
        self.0
            .send(Command::ProvideSegment { segment_id, data })
            .await?;
        Ok(())
    }

    /// Request the content of the given file from the given peer.
    pub async fn request_segment(&mut self, segment_id: String) -> Result<Uint8Array, ClientError> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(Command::RequestSegment { segment_id, sender })
            .await?;

        let segment = receiver.await??;
        let buf = Uint8Array::from(segment.as_slice());

        Ok(buf)
    }

    pub async fn quit(&mut self) -> Result<(), ClientError> {
        self.0.send(Command::Quit).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ClientError {
    ConfigError,
    ListenError,
    BadNamespace,
    DialError,
    ConnectionClosed,
    RequestError(RequestError),
}

impl From<ClientError> for wasm_bindgen::JsValue {
    fn from(val: ClientError) -> Self {
        match val {
            ClientError::ConfigError => 3.into(),
            ClientError::BadNamespace => 0.into(),
            ClientError::ListenError => 4.into(),
            ClientError::DialError => 5.into(),
            ClientError::ConnectionClosed => 1.into(),
            ClientError::RequestError(_) => 2.into(),
        }
    }
}

impl From<NamespaceTooLong> for ClientError {
    fn from(value: NamespaceTooLong) -> Self {
        ClientError::BadNamespace
    }
}

impl From<SendError> for ClientError {
    fn from(_: SendError) -> Self {
        ClientError::ConnectionClosed
    }
}

impl From<Canceled> for ClientError {
    fn from(_: Canceled) -> Self {
        ClientError::ConnectionClosed
    }
}

impl From<RequestError> for ClientError {
    fn from(err: RequestError) -> Self {
        ClientError::RequestError(err)
    }
}

impl From<DialError> for ClientError {
    fn from(value: DialError) -> Self {
        ClientError::DialError
    }
}
