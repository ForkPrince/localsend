//! Optional backend discovery and relay.
//!
//! A device connects to a backend server over WebSocket and joins a room
//! (derived from a user-chosen secret). The backend keeps a directory of the
//! online devices in each room and can bridge any two of them: an `Open`
//! request from one device is answered on the other side with an `Incoming`
//! event, after which the backend pipes the bytes of a session between the two
//! sockets. This is how a device that is not directly reachable (e.g. behind a
//! NAT) can both be discovered and receive data.
//!
//! The relay only ever forwards raw bytes; the LocalSend protocol (including
//! the end-to-end TLS handshake and the certificate-pinned identity) runs
//! unchanged between the two devices, so the backend cannot read or alter the
//! content it relays.
//!
//! Two entry points are provided:
//! - [`RelayClient::open_proxy`] lets an HTTP client send through the relay by
//!   dialing a local address instead of the peer's real one.
//! - incoming sessions arrive as [`RelayEvent::Incoming`] carrying a
//!   [`RelayPipe`], which is fed into the HTTP server like a regular TCP
//!   connection.
//!
//! Devices announced by the backend can be turned into [`crate::discovery::DiscoveredDevice`]s
//! (see [`RelayDeviceInfo`]) and put into the discovery store, so they appear
//! on the send tab.

mod client;
mod messages;
mod pipe;

pub use client::{RelayClient, RelayEvent};
pub use messages::{
    room_key, RelayClientMessage, RelayDeviceInfo, RelayPeer, RelayServerMessage,
};
pub use pipe::RelayPipe;