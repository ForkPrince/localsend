use crate::frb_generated::StreamSink;
use localsend::model::discovery::DeviceType;
use localsend::relay::{
    RelayClient as CoreRelayClient, RelayDeviceInfo, RelayPeer,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Information a device announces about itself to the relay backend.
///
/// A subset of the v2 `RegisterDtoV2`, without the address-dependent fields.
#[derive(Clone)]
pub struct RsRelayInfo {
    pub alias: String,
    pub version: String,
    pub device_model: Option<String>,
    pub device_type: Option<DeviceType>,
    pub fingerprint: String,
    pub cert_fingerprint: Option<String>,
    pub download: bool,
}

impl From<RsRelayInfo> for RelayDeviceInfo {
    fn from(info: RsRelayInfo) -> Self {
        RelayDeviceInfo {
            alias: info.alias,
            version: info.version,
            device_model: info.device_model,
            device_type: info.device_type,
            fingerprint: info.fingerprint,
            cert_fingerprint: info.cert_fingerprint,
            download: info.download,
        }
    }
}

/// A peer announced by the relay backend, as presented to this device.
/// A peer announced by the relay backend, as presented to this device.
#[derive(Clone)]
pub struct RsRelayPeer {
    pub client_id: String,
    pub info: RsRelayInfo,
}

impl From<&RelayPeer> for RsRelayPeer {
    fn from(peer: &RelayPeer) -> Self {
        RsRelayPeer {
            client_id: peer.client_id.to_string(),
            info: RsRelayInfo {
                alias: peer.info.alias.clone(),
                version: peer.info.version.clone(),
                device_model: peer.info.device_model.clone(),
                device_type: peer.info.device_type.clone(),
                fingerprint: peer.info.fingerprint.clone(),
                cert_fingerprint: peer.info.cert_fingerprint.clone(),
                download: peer.info.download,
            },
        }
    }
}

/// An event emitted by an [RsRelayClient].
#[derive(Clone)]
pub enum RsRelayEvent {
    /// The connection is established and this device has an ID.
    Connected { client_id: String },

    /// A device joined the room, or is currently in the room (on connect).
    Peer { peer: RsRelayPeer },

    /// A device updated its information.
    PeerUpdate { peer: RsRelayPeer },

    /// A device left the room.
    PeerLeft { peer_id: String },

    /// The connection was lost; the client must be reconnected.
    Disconnected { error: Option<String> },
}

/// A handle to a connected relay client: the outbound WebSocket to the backend
/// that both discovers the peers in a room and opens relay sessions to them.
///
/// Incoming relay sessions are fed into the HTTP server by the task started in
/// `RsHttpServer::start_relay`; the event stream surfaces the peers so the app
/// can add them to the discovery store.
pub struct RsRelayClient {
    pub(crate) inner: Arc<CoreRelayClient>,
    event_rx: Mutex<Option<tokio::sync::mpsc::Receiver<RsRelayEvent>>>,
}

impl RsRelayClient {
    pub(crate) fn new(
        inner: Arc<CoreRelayClient>,
        event_rx: tokio::sync::mpsc::Receiver<RsRelayEvent>,
    ) -> Self {
        RsRelayClient {
            inner,
            event_rx: Mutex::new(Some(event_rx)),
        }
    }

    /// The ID the backend assigned to this device, once connected.
    pub async fn client_id(&self) -> Option<String> {
        self.inner.client_id().await.map(|id| id.to_string())
    }

    /// Emits [RsRelayEvent]s until the connection is lost or the client is
    /// stopped. Can only be listened to once.
    pub async fn listen(&self, sink: StreamSink<RsRelayEvent>) {
        let Some(mut event_rx) = self.event_rx.lock().await.take() else {
            let _ = sink.add_error(anyhow::anyhow!("Relay events already listened to"));
            return;
        };

        while let Some(event) = event_rx.recv().await {
            if sink.add(event).is_err() {
                break;
            }
        }
    }

    /// Updates the announced device information.
    pub async fn update_info(&self, info: RsRelayInfo) -> anyhow::Result<()> {
        self.inner
            .update_info(info.into())
            .await
            .map_err(|err| anyhow::anyhow!("{err:#}"))
    }

    /// Opens a relay session to the device `target_id` for every connection
    /// accepted on a local TCP listener, and bridges the bytes between the two.
    /// Returns the local address to dial.
    ///
    /// This is how a relayed HTTP transfer is sent: the HTTP client connects
    /// to the returned address instead of the peer's real address, and the
    /// TLS handshake travels through the backend to the peer.
    pub async fn open_proxy(&self, target_id: String) -> anyhow::Result<String> {
        let target_id =
            Uuid::parse_str(&target_id).map_err(|_| anyhow::anyhow!("Invalid relay target id"))?;
        Ok(self.inner.open_proxy(target_id).await?.to_string())
    }

    /// Stops the client: closes the websocket and ends every session.
    pub async fn stop(&self) {
        self.inner.stop().await;
    }
}
