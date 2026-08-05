//! The relay backend endpoint: a per-room directory of connected devices and a
//! transparent byte bridge between any two of them.
//!
//! Unlike the WebRTC signaling endpoint, the relay never interprets the bytes
//! it forwards. Devices in the same room exchange control messages as JSON
//! text, and once a session is opened the session bytes travel as binary frames
//! (`[session_id (16 bytes)][payload]`, see `localsend::relay::encode_data`).
//! The backend only routes those frames to the peer of the session; the
//! LocalSend protocol, TLS handshake and end-to-end identity run between the
//! two devices themselves.

use crate::config::error::AppError;
use crate::config::state::{AppState, RelayClient, RelayOutbound, RelaySession};
use crate::util::ip::{client_ip, get_ip_group};
use axum::body::Body;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use localsend::relay::{decode_data, RelayClientMessage, RelayDeviceInfo, RelayPeer, RelayServerMessage};
use std::net::SocketAddr;
use std::sync::LazyLock;
use tokio::sync::mpsc;
use uuid::Uuid;

/// How many devices a relay room may hold, mirroring `MAX_CONNECTIONS_PER_IP`
/// of the WebRTC signaling endpoint.
static MAX_CONNECTIONS_PER_ROOM: LazyLock<usize> = LazyLock::new(|| {
    std::env::var("MAX_RELAY_CONNECTIONS_PER_ROOM")
        .unwrap_or_else(|_| "128".to_string())
        .parse::<usize>()
        .unwrap()
});

type RelayTx = mpsc::Sender<RelayOutbound>;

pub(crate) async fn relay_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Response<Body>, AppError> {
    let ip_group = get_ip_group(client_ip(&headers, &addr));
    Ok(ws.on_upgrade(move |socket| handle_socket(state, socket, ip_group)))
}

fn encode_control(msg: &RelayServerMessage) -> Message {
    let json = serde_json::to_string(msg).unwrap_or_else(|_| "{}".to_string());
    Message::Text(json.into())
}

async fn handle_socket(state: AppState, socket: WebSocket, ip_group: String) {
    let (mut sender, mut receiver) = socket.split();

    // The first message must be `Hello`, announcing the room and the device.
    let hello: Option<RelayClientMessage> = match receiver.next().await {
        Some(Ok(Message::Text(text))) => serde_json::from_str(&text).ok(),
        _ => None,
    };
    let (room, info): (String, RelayDeviceInfo) = match hello {
        Some(RelayClientMessage::Hello { room, info }) => (room, info),
        _ => {
            let _ = sender.close().await;
            return;
        }
    };

    let client_id = Uuid::new_v4();
    let (tx, mut rx) = mpsc::channel::<RelayOutbound>(64);

    // Join the room (unless it is full), announce the device to its peers and
    // collect the peers for the `Welcome`.
    let peers = {
        let mut store = state.relay_state.lock().await;

        if store.rooms.get(&room).is_some_and(|r| r.len() >= *MAX_CONNECTIONS_PER_ROOM) {
            let _ = tx.send(RelayOutbound::Control(RelayServerMessage::Error { code: 503 })).await;
            drop(store);
            let _ = sender.close().await;
            return;
        }
        if crate::util::limit::rate_limit(&ip_group, &state.request_count_map).await.is_err() {
            let _ = tx.send(RelayOutbound::Control(RelayServerMessage::Error { code: 429 })).await;
            drop(store);
            let _ = sender.close().await;
            return;
        }

        let entry = store.rooms.entry(room.clone()).or_default();
        let peers: Vec<RelayPeer> = entry
            .iter()
            .map(|(id, c)| RelayPeer { client_id: *id, info: c.info.clone() })
            .collect();
        let siblings: Vec<RelayTx> = entry.values().map(|c| c.tx.clone()).collect();
        entry.insert(client_id, RelayClient { info: info.clone(), tx: tx.clone() });
        drop(store);

        let self_peer = RelayPeer { client_id, info: info.clone() };
        for sibling in siblings {
            let _ = sibling
                .send(RelayOutbound::Control(RelayServerMessage::PeerJoined { peer: self_peer.clone() }))
                .await;
        }
        peers
    };

    let _ = tx
        .send(RelayOutbound::Control(RelayServerMessage::Welcome { client_id, peers }))
        .await;
    drop(tx);

    let mut send_task = tokio::spawn(async move {
        while let Some(out) = rx.recv().await {
            let frame = match out {
                RelayOutbound::Control(msg) => encode_control(&msg),
                RelayOutbound::Data(frame) => Message::Binary(frame.into()),
            };
            if sender.send(frame).await.is_err() {
                break;
            }
        }
    });

    let room_loop = room.clone();
    let state_recv = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    let Ok(cmsg) = serde_json::from_str::<RelayClientMessage>(&text) else {
                        continue;
                    };
                    if crate::util::limit::rate_limit(&ip_group, &state_recv.request_count_map).await.is_err() {
                        break;
                    }
                    match cmsg {
                        RelayClientMessage::Hello { .. } => {}
                        RelayClientMessage::Update { info } => {
                            update_peer_info(&state_recv, &room_loop, client_id, info).await;
                        }
                        RelayClientMessage::Open { session_id, target_id } => {
                            open_session(&state_recv, &room_loop, client_id, target_id, session_id).await;
                        }
                        RelayClientMessage::Close { session_id } => {
                            close_session(&state_recv, client_id, session_id).await;
                        }
                    }
                }
                Message::Binary(frame) => {
                    if let Some((session_id, _)) = decode_data(&frame) {
                        route_data(&state_recv, client_id, session_id, frame.to_vec()).await;
                    }
                }
                _ => break,
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => { recv_task.abort(); }
        _ = &mut recv_task => { send_task.abort(); }
    }
    send_task.abort();
    recv_task.abort();

    // The socket is gone: deregister the device and notify its sessions and the
    // other ends of those sessions.
    let (peers, closed_sessions) = leave_room(&state, &room, client_id).await;
    for peer_tx in peers {
        let _ = peer_tx
            .send(RelayOutbound::Control(RelayServerMessage::PeerLeft { peer_id: client_id }))
            .await;
    }
    for (session_id, other) in closed_sessions {
        let _ = other
            .send(RelayOutbound::Control(RelayServerMessage::Closed { session_id }))
            .await;
    }
    tracing::info!("Relay disconnect: {room} / {client_id}");
}

async fn update_peer_info(state: &AppState, room: &str, client_id: Uuid, info: RelayDeviceInfo) {
    let mut store = state.relay_state.lock().await;
    let Some(entry) = store.rooms.get_mut(room) else { return };
    let Some(client) = entry.get_mut(&client_id) else { return };
    client.info = info.clone();
    let peer = RelayPeer { client_id, info };
    let siblings: Vec<RelayTx> = entry
        .iter()
        .filter(|(id, _)| *id != &client_id)
        .map(|(_, c)| c.tx.clone())
        .collect();
    drop(store);
    for s in siblings {
        let _ = s.send(RelayOutbound::Control(RelayServerMessage::PeerUpdate { peer: peer.clone() })).await;
    }
}

enum SessionOutcome {
    Opened { peer: RelayPeer, target_tx: RelayTx, initiator_tx: RelayTx },
    Unreachable { initiator_tx: RelayTx },
}

async fn open_session(
    state: &AppState,
    room: &str,
    initiator_id: Uuid,
    target_id: Uuid,
    session_id: Uuid,
) {
    let outcome = plan_open(state, room, initiator_id, target_id, session_id).await;
    let Some(outcome) = outcome else { return };

    match outcome {
        SessionOutcome::Opened { peer, target_tx, initiator_tx } => {
            let _ = target_tx
                .send(RelayOutbound::Control(RelayServerMessage::Incoming { session_id, peer }))
                .await;
            let _ = initiator_tx
                .send(RelayOutbound::Control(RelayServerMessage::Opened { session_id }))
                .await;
        }
        SessionOutcome::Unreachable { initiator_tx } => {
            let _ = initiator_tx
                .send(RelayOutbound::Control(RelayServerMessage::Closed { session_id }))
                .await;
        }
    }
}

async fn plan_open(
    state: &AppState,
    room: &str,
    initiator_id: Uuid,
    target_id: Uuid,
    session_id: Uuid,
) -> Option<SessionOutcome> {
    let (initiator_tx, target_tx, initiator_info) = {
        let store = state.relay_state.lock().await;
        let entry = store.rooms.get(room)?;
        let initiator = entry.get(&initiator_id)?;
        let target_tx = entry.get(&target_id).map(|target| target.tx.clone());
        (initiator.tx.clone(), target_tx, initiator.info.clone())
    };
    let Some(target_tx) = target_tx else {
        return Some(SessionOutcome::Unreachable { initiator_tx });
    };

    {
        let mut store = state.relay_state.lock().await;
        store.sessions.insert(
            session_id,
            RelaySession {
                a_id: initiator_id,
                a_tx: initiator_tx.clone(),
                b_id: target_id,
                b_tx: target_tx.clone(),
            },
        );
    }

    Some(SessionOutcome::Opened {
        peer: RelayPeer { client_id: initiator_id, info: initiator_info },
        target_tx,
        initiator_tx,
    })
}

async fn route_data(state: &AppState, source_id: Uuid, session_id: Uuid, frame: Vec<u8>) {
    let other = {
        let store = state.relay_state.lock().await;
        store.sessions.get(&session_id).and_then(|s| {
            if s.a_id == source_id {
                Some(s.b_tx.clone())
            } else if s.b_id == source_id {
                Some(s.a_tx.clone())
            } else {
                None
            }
        })
    };
    if let Some(tx) = other {
        let _ = tx.send(RelayOutbound::Data(frame)).await;
    }
}

async fn close_session(state: &AppState, source_id: Uuid, session_id: Uuid) {
    let other = {
        let mut store = state.relay_state.lock().await;
        match store.sessions.remove(&session_id) {
            Some(s) if s.a_id == source_id => Some(s.b_tx),
            Some(s) if s.b_id == source_id => Some(s.a_tx),
            _ => None,
        }
    };
    if let Some(tx) = other {
        let _ = tx.send(RelayOutbound::Control(RelayServerMessage::Closed { session_id })).await;
    }
}

async fn leave_room(
    state: &AppState,
    room: &str,
    client_id: Uuid,
) -> (Vec<RelayTx>, Vec<(Uuid, RelayTx)>) {
    let mut peers = Vec::new();
    let mut closed = Vec::new();
    {
        let mut store = state.relay_state.lock().await;
        store.sessions.retain(|session_id, s| {
            if s.a_id == client_id {
                closed.push((*session_id, s.b_tx.clone()));
                false
            } else if s.b_id == client_id {
                closed.push((*session_id, s.a_tx.clone()));
                false
            } else {
                true
            }
        });
        if let Some(entry) = store.rooms.get_mut(room) {
            if entry.remove(&client_id).is_some() {
                peers = entry.values().map(|c| c.tx.clone()).collect();
            }
            if entry.is_empty() {
                store.rooms.remove(room);
            }
        }
    }
    (peers, closed)
}