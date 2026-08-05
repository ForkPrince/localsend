//! The client of the relay backend: discovery and outbound sessions.

use super::messages::{
    decode_data, encode_data, room_key, RelayClientMessage, RelayDeviceInfo, RelayPeer,
    RelayServerMessage,
};
use super::pipe::{PipeInbound, PipeOutbound, RelayPipe};
use anyhow::{anyhow, Context as _};
use futures_util::{SinkExt, Stream, StreamExt};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tungstenite::{Bytes, Message};
use uuid::Uuid;

/// How long a session `Open` request may take to be confirmed by the backend.
const OPEN_TIMEOUT: Duration = Duration::from_secs(10);

/// How big the data chunks read from a pipe's outbound side are.
const CHUNK_SIZE: usize = 32 * 1024;

/// The capacity of the internal websocket write queue.
const WS_CHANNEL_SIZE: usize = 64;

/// An event emitted by a [`RelayClient`].
pub enum RelayEvent {
    /// The connection is established and this device has an ID.
    Connected { client_id: Uuid },

    /// A device joined the room, or is currently in the room (on connect).
    Peer { peer: RelayPeer },

    /// A device updated its information.
    PeerUpdate { peer: RelayPeer },

    /// A device left the room.
    PeerLeft { peer_id: Uuid },

    /// A peer in the room opened a relay session to this device. The caller
    /// feeds `pipe` into the HTTP server to receive its requests.
    Incoming {
        session_id: Uuid,
        peer: RelayPeer,
        pipe: RelayPipe,
    },

    /// The connection was lost; the client must be reconnected.
    Disconnected { error: Option<String> },
}

impl std::fmt::Display for RelayEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayEvent::Connected { client_id } => {
                write!(f, "Connected ({client_id})")
            }
            RelayEvent::Peer { peer } => write!(f, "Peer joined: {}", peer.info.alias),
            RelayEvent::PeerUpdate { peer } => write!(f, "Peer updated: {}", peer.info.alias),
            RelayEvent::PeerLeft { peer_id } => write!(f, "Peer left ({peer_id})"),
            RelayEvent::Incoming {
                session_id,
                peer,
                pipe: _,
            } => write!(f, "Incoming session {session_id} from {}", peer.info.alias),
            RelayEvent::Disconnected { error } => {
                write!(f, "Disconnected ({error:?})")
            }
        }
    }
}

/// What is written to the backend: a control message, or the bytes of a relay
/// session (serialized as a binary frame).
enum WsOut {
    Control(RelayClientMessage),
    Data { session_id: Uuid, bytes: Vec<u8> },
}

/// Per-session state kept by the client: the end that feeds the bytes received
/// from the other end into the pipe (and drives its EOF), and a token that
/// cancels the session's outbound drainer.
struct SessionState {
    inbound: PipeInbound,
    cancel: CancellationToken,
}

struct RelayClientInner {
    /// The channel to the websocket writer task.
    ws_tx: mpsc::Sender<WsOut>,

    /// Per-session state, keyed by session ID.
    sessions: Mutex<HashMap<Uuid, SessionState>>,

    /// The senders that complete a pending [`RelayClient::open_session`].
    opened: Mutex<HashMap<Uuid, oneshot::Sender<Result<(), ()>>>>,

    /// The id the backend assigned to this device.
    client_id: RwLock<Option<Uuid>>,

    /// Channel on which events are emitted.
    event_tx: mpsc::Sender<RelayEvent>,
}

/// A handle to a connected relay client.
#[derive(Clone)]
pub struct RelayClient {
    inner: Arc<RelayClientInner>,
}

impl RelayClient {
    /// Connects to the relay backend at `url` and joins the room derived from
    /// `room_secret`. Returns the client and the channel of emitted events.
    ///
    /// The connection is established asynchronously:
    /// [`RelayEvent::Connected`] is emitted once the backend answered.
    pub async fn connect(
        url: &str,
        room_secret: &str,
        info: RelayDeviceInfo,
    ) -> anyhow::Result<(RelayClient, mpsc::Receiver<RelayEvent>)> {
        let (event_tx, event_rx) = mpsc::channel(64);
        let (ws_tx, mut ws_rx) = mpsc::channel(WS_CHANNEL_SIZE);

        let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
        let (ws_write, ws_read) = ws_stream.split();

        // Writer task: serializes control and data messages into frames.
        tokio::spawn(async move {
            let mut ws_write = ws_write;
            while let Some(out) = ws_rx.recv().await {
                let frame = match out {
                    WsOut::Control(message) => {
                        let json = match serde_json::to_string(&message) {
                            Ok(json) => json,
                            Err(err) => {
                                tracing::warn!("Could not serialize relay message: {err:#}");
                                continue;
                            }
                        };
                        Message::Text(json.into())
                    }
                    WsOut::Data { session_id, bytes } => {
                        Message::Binary(Bytes::from(encode_data(session_id, &bytes)))
                    }
                };
                if ws_write.send(frame).await.is_err() {
                    break;
                }
            }
        });

        let inner = Arc::new(RelayClientInner {
            ws_tx,
            sessions: Mutex::new(HashMap::new()),
            opened: Mutex::new(HashMap::new()),
            client_id: RwLock::new(None),
            event_tx,
        });

        inner
            .ws_tx
            .send(WsOut::Control(RelayClientMessage::Hello {
                room: room_key(room_secret),
                info,
            }))
            .await
            .context("Relay connection closed while joining the room")?;

        let receive_inner = inner.clone();
        tokio::spawn(receive_loop(receive_inner, ws_read));

        Ok((RelayClient { inner }, event_rx))
    }

    /// The ID the backend assigned to this device, once connected.
    pub async fn client_id(&self) -> Option<Uuid> {
        *self.inner.client_id.read().await
    }

    /// Updates the announced device information.
    pub async fn update_info(&self, info: RelayDeviceInfo) -> anyhow::Result<()> {
        self.inner
            .ws_tx
            .send(WsOut::Control(RelayClientMessage::Update { info }))
            .await
            .map_err(|_| anyhow!("Relay connection closed"))
    }

    /// Opens a relay session to the device `target_id` and returns the pipe to
    /// transfer bytes through. The session is established once the backend
    /// confirmed that the target is connected.
    pub async fn open_session(&self, target_id: Uuid) -> anyhow::Result<RelayPipe> {
        let session_id = Uuid::new_v4();
        let (pipe, inbound, outbound) = RelayPipe::new();

        Self::register_session(&self.inner, session_id, inbound, outbound).await;

        let (tx, rx) = oneshot::channel();
        self.inner.opened.lock().await.insert(session_id, tx);

        self.inner
            .ws_tx
            .send(WsOut::Control(RelayClientMessage::Open {
                session_id,
                target_id,
            }))
            .await
            .map_err(|_| anyhow!("Relay connection closed"))?;

        let confirmed = timeout(OPEN_TIMEOUT, rx)
            .await
            .context("Timed out waiting for the relay session to open")?;
        anyhow::ensure!(confirmed.is_ok(), "The relay target is not reachable");

        Ok(pipe)
    }

    /// Opens a relay session to the device `target_id` for every connection
    /// accepted on a local TCP listener, and bridges the bytes between the two.
    /// Returns the local address to dial.
    ///
    /// This is how a relayed HTTP transfer is sent: the HTTP client connects
    /// to the returned address instead of the peer's real address, and the
    /// TLS handshake travels through the backend to the peer.
    pub async fn open_proxy(&self, target_id: Uuid) -> anyhow::Result<SocketAddr> {
        let listener =
            TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)).await?;
        let local_addr = listener.local_addr()?;
        let client = self.clone();
        tokio::spawn(async move {
            loop {
                let Ok((tcp, _)) = listener.accept().await else {
                    break;
                };
                let client = client.clone();
                tokio::spawn(async move {
                    bridge_tcp_to_relay(tcp, client, target_id).await;
                });
            }
        });
        Ok(local_addr)
    }

    /// Registers a new session: stores its state and spawns the task that
    /// drains the pipe's outbound side into the backend.
    async fn register_session(
        inner: &Arc<RelayClientInner>,
        session_id: Uuid,
        inbound: PipeInbound,
        outbound: PipeOutbound,
    ) {
        let cancel = CancellationToken::new();
        inner
            .sessions
            .lock()
            .await
            .insert(
                session_id,
                SessionState {
                    inbound,
                    cancel: cancel.clone(),
                },
            );
        let ws_tx = inner.ws_tx.clone();
        tokio::spawn(drain_outbound(session_id, ws_tx, cancel, outbound));
    }
}

/// Bridges one local TCP connection to a relay session.
async fn bridge_tcp_to_relay(tcp: TcpStream, client: RelayClient, target_id: Uuid) {
    let mut pipe = match client.open_session(target_id).await {
        Ok(pipe) => pipe,
        Err(err) => {
            tracing::debug!("Could not open relay session: {err:#}");
            return;
        }
    };
    let mut tcp = tcp;
    let _ = tokio::io::copy_bidirectional(&mut tcp, &mut pipe).await;
}

/// Reads the outbound half of a session and sends its bytes to the backend as
/// binary frames, until the session is closed or the data ends.
async fn drain_outbound(
    session_id: Uuid,
    ws_tx: mpsc::Sender<WsOut>,
    cancel: CancellationToken,
    mut reader: PipeOutbound,
) {
    let mut chunk = vec![0u8; CHUNK_SIZE];
    loop {
        let result = tokio::select! {
            _ = cancel.cancelled() => break,
            r = reader.read(&mut chunk) => r,
        };
        match result {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if ws_tx
                    .send(WsOut::Data {
                        session_id,
                        bytes: chunk[..n].to_vec(),
                    })
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    }
    let _ = ws_tx
        .send(WsOut::Control(RelayClientMessage::Close { session_id }))
        .await;
}

/// Runs the websocket receive loop, routing control messages to the event
/// channel and binary frames to the matching session pipe.
async fn receive_loop(
    inner: Arc<RelayClientInner>,
    mut ws_read: impl Stream<Item = Result<Message, tungstenite::Error>> + Unpin,
) {
    let event_tx = inner.event_tx.clone();
    while let Some(message) = ws_read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let message = match serde_json::from_str::<RelayServerMessage>(&text) {
                    Ok(message) => message,
                    Err(err) => {
                        tracing::warn!("Could not parse relay message: {err:#}");
                        continue;
                    }
                };
                match message {
                    RelayServerMessage::Welcome { client_id, peers } => {
                        *inner.client_id.write().await = Some(client_id);
                        for peer in peers {
                            let _ = event_tx.send(RelayEvent::Peer { peer }).await;
                        }
                        let _ = event_tx.send(RelayEvent::Connected { client_id }).await;
                    }
                    RelayServerMessage::PeerJoined { peer } => {
                        let _ = event_tx.send(RelayEvent::Peer { peer }).await;
                    }
                    RelayServerMessage::PeerUpdate { peer } => {
                        let _ = event_tx.send(RelayEvent::PeerUpdate { peer }).await;
                    }
                    RelayServerMessage::PeerLeft { peer_id } => {
                        let _ = event_tx.send(RelayEvent::PeerLeft { peer_id }).await;
                    }
                    RelayServerMessage::Incoming { session_id, peer } => {
                        handle_incoming(&inner, session_id, peer).await;
                    }
                    RelayServerMessage::Opened { session_id } => {
                        if let Some(tx) = inner.opened.lock().await.remove(&session_id) {
                            let _ = tx.send(Ok(()));
                        }
                    }
                    RelayServerMessage::Closed { session_id } => {
                        close_session(&inner, session_id).await;
                    }
                    RelayServerMessage::Error { code } => {
                        tracing::warn!("Relay backend error: {code}");
                    }
                }
            }
            Ok(Message::Binary(frame)) => {
                if let Some((session_id, payload)) = decode_data(&frame) {
                    inject_data(&inner, session_id, payload).await;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(err) => {
                tracing::debug!("Relay websocket error: {err:#}");
                break;
            }
        }
    }

    // The connection is gone: signal EOF on every pipe and fail pending opens.
    for (session_id, state) in inner.sessions.lock().await.drain() {
        state.cancel.cancel();
        state.inbound.close();
        if let Some(tx) = inner.opened.lock().await.remove(&session_id) {
            let _ = tx.send(Err(()));
        }
    }
    let _ = event_tx
        .send(RelayEvent::Disconnected { error: None })
        .await;
}

/// An inbound session opened by a peer: creates the pipe and emits it.
async fn handle_incoming(inner: &Arc<RelayClientInner>, session_id: Uuid, peer: RelayPeer) {
    let (pipe, inbound, outbound) = RelayPipe::new();
    RelayClient::register_session(inner, session_id, inbound, outbound).await;
    let _ = inner
        .event_tx
        .send(RelayEvent::Incoming { session_id, peer, pipe })
        .await;
}

/// Feeds the bytes of a binary frame into the pipe's read side.
async fn inject_data(inner: &RelayClientInner, session_id: Uuid, payload: &[u8]) {
    let sessions = inner.sessions.lock().await;
    if let Some(state) = sessions.get(&session_id) {
        state.inbound.push(payload);
    }
}

/// Closes a session: cancels its outbound drainer and EOFs its pipe.
async fn close_session(inner: &RelayClientInner, session_id: Uuid) {
    if let Some(state) = inner.sessions.lock().await.remove(&session_id) {
        state.cancel.cancel();
        state.inbound.close();
    }
    if let Some(tx) = inner.opened.lock().await.remove(&session_id) {
        let _ = tx.send(Err(()));
    }
}
