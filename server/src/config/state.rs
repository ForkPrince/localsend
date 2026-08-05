use localsend::relay::{RelayDeviceInfo, RelayServerMessage};
use localsend::webrtc::signaling::{ClientInfoWithoutId, WsServerMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

/// IP -> Peer ID -> PeerInfo + WebSocket message sender.
pub type TxMap = Arc<Mutex<HashMap<String, HashMap<Uuid, ClientState>>>>;

pub struct ClientState {
    pub client: ClientInfoWithoutId,
    pub tx: mpsc::Sender<WsServerMessage>,
}

pub type IpRequestCountMap = Arc<Mutex<HashMap<String, u32>>>;

/// An outbound message of a relay device's socket: a control message encoded
/// as JSON text, or the raw bytes of a relay session sent as a binary frame.
pub enum RelayOutbound {
    Control(RelayServerMessage),
    Data(Vec<u8>),
}

pub type RelayTx = mpsc::Sender<RelayOutbound>;

/// A device connected to the relay backend, in a room.
pub struct RelayClient {
    pub info: RelayDeviceInfo,
    pub tx: RelayTx,
}

/// A live relay session: a bidirectional pipe between two devices in a room.
pub struct RelaySession {
    pub a_id: Uuid,
    pub a_tx: RelayTx,
    pub b_id: Uuid,
    pub b_tx: RelayTx,
}

/// The relay backend state: the per-room device directory and the live
/// sessions that the backend pipes bytes through.
#[derive(Default)]
pub struct RelayStore {
    /// Room key -> devices currently connected.
    pub rooms: HashMap<String, HashMap<Uuid, RelayClient>>,

    /// Session ID -> the two ends of the tunnel.
    pub sessions: HashMap<Uuid, RelaySession>,
}

pub type RelayState = Arc<Mutex<RelayStore>>;

#[derive(Clone)]
pub struct AppState {
    /// Map of peer IDs to WebSocket message senders.
    pub tx_map: TxMap,

    /// Map of IP addresses to the number of requests.
    pub request_count_map: IpRequestCountMap,

    /// Relay rooms and sessions.
    pub relay_state: RelayState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tx_map: Arc::new(Mutex::new(HashMap::new())),
            request_count_map: Arc::new(Mutex::new(HashMap::new())),
            relay_state: Arc::new(Mutex::new(RelayStore::default())),
        }
    }
}
