//! The bidirectional byte stream of a relay session.

use bytes::BytesMut;
use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use tokio::io::{AsyncRead, AsyncWrite};

/// How much data a relay pipe buffers in each direction before applying
/// backpressure. Large enough that the TLS and HTTP layers never stall on
/// small bursts, small enough that a slow peer is throttled quickly.
const PIPE_BUFFER: usize = 256 * 1024;

/// The state shared by the two ends of a relay session.
struct Shared {
    /// Bytes received from the other end, pending to be read by the HTTP side.
    inbound: Mutex<BytesMut>,
    inbound_closed: AtomicBool,
    inbound_waker: Mutex<Option<Waker>>,

    /// Bytes written by the HTTP side, pending to be sent to the other end.
    outbound: Mutex<BytesMut>,
    outbound_closed: AtomicBool,
    outbound_reader_waker: Mutex<Option<Waker>>,
    outbound_writer_waker: Mutex<Option<Waker>>,
}

/// A bidirectional byte stream through the relay backend, over one session.
///
/// It behaves like a `TcpStream` (it implements [`AsyncRead`] and [`AsyncWrite`]), so the
/// HTTP server and client can use it transparently, TLS handshake included. The backend only
/// ever sees the raw bytes flowing through it.
pub struct RelayPipe {
    state: Arc<Shared>,
}

/// The end of a session driven by the backend receive loop: it feeds the bytes received from
/// the other end into the pipe and signals EOF to it.
pub struct PipeInbound {
    state: Arc<Shared>,
}

/// The end of a pipe that the backend send loop reads: it reads the bytes the HTTP side writes
/// to the pipe, to be sent to the other end.
pub struct PipeOutbound {
    state: Arc<Shared>,
}

impl RelayPipe {
    /// Creates a pipe together with the two ends that drive it. The HTTP side uses the returned
    /// [`RelayPipe`]; [`PipeInbound`] pushes the bytes received from the other end into it, and
    /// [`PipeOutbound`] yields the bytes that are written to it (to send to the other end).
    pub fn new() -> (Self, PipeInbound, PipeOutbound) {
        let shared = Arc::new(Shared {
            inbound: Mutex::new(BytesMut::new()),
            inbound_closed: AtomicBool::new(false),
            inbound_waker: Mutex::new(None),
            outbound: Mutex::new(BytesMut::new()),
            outbound_closed: AtomicBool::new(false),
            outbound_reader_waker: Mutex::new(None),
            outbound_writer_waker: Mutex::new(None),
        });
        (
            RelayPipe { state: shared.clone() },
            PipeInbound { state: shared.clone() },
            PipeOutbound { state: shared.clone() },
        )
    }
}

impl PipeInbound {
    /// Feeds the bytes received from the other end into the pipe.
    pub fn push(&self, bytes: &[u8]) {
        let mut inbound = match self.state.inbound.lock() {
            Ok(guard) => guard,
            Err(err) => {
                tracing::debug!("Relay pipe poisoned: {err}");
                return;
            }
        };
        inbound.extend_from_slice(bytes);
        if let Ok(mut waker) = self.state.inbound_waker.lock() {
            if let Some(waker) = waker.take() {
                waker.wake();
            }
        }
    }

    /// Signals EOF to the HTTP side, so a blocked read returns.
    pub fn close(&self) {
        self.state.inbound_closed.store(true, Ordering::SeqCst);
        if let Ok(mut waker) = self.state.inbound_waker.lock() {
            if let Some(waker) = waker.take() {
                waker.wake();
            }
        }
    }
}

impl AsyncRead for RelayPipe {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        read_one(&self.state.inbound, &self.state.inbound_closed, &self.state.inbound_waker, cx, buf)
    }
}

impl AsyncWrite for RelayPipe {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        write_one(&self.state.outbound, &self.state.outbound_closed, &self.state.outbound_reader_waker, &self.state.outbound_writer_waker, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        // Nothing is buffered in an underlying writer; data is drained
        // asynchronously by the `PipeOutbound` reader. Re-arm if full.
        wake_backpressure(&self.state.outbound, &self.state.outbound_writer_waker, cx);
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.state.outbound_closed.store(true, Ordering::SeqCst);
        wake_readers(&self.state.outbound_reader_waker);
        wake_backpressure(&self.state.outbound, &self.state.outbound_writer_waker, cx);
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for PipeOutbound {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        read_one(&self.state.outbound, &self.state.outbound_closed, &self.state.outbound_reader_waker, cx, buf)
    }
}

/// The shared read logic: take bytes out of `buffer` (or report EOF).
fn read_one(
    buffer: &Mutex<BytesMut>,
    closed: &AtomicBool,
    waker: &Mutex<Option<Waker>>,
    cx: &mut Context<'_>,
    buf: &mut tokio::io::ReadBuf<'_>,
) -> Poll<io::Result<()>> {
    let mut bytes = buffer.lock().unwrap();
    if !bytes.is_empty() {
        let n = buf.remaining().min(bytes.len());
        buf.put_slice(&bytes.split_to(n));
        Poll::Ready(Ok(()))
    } else if closed.load(Ordering::SeqCst) {
        Poll::Ready(Ok(()))
    } else {
        *waker.lock().unwrap() = Some(cx.waker().clone());
        Poll::Pending
    }
}

/// The shared write logic: append to the outbound buffer, applying backpressure.
fn write_one(
    buffer: &Mutex<BytesMut>,
    closed: &AtomicBool,
    reader_waker: &Mutex<Option<Waker>>,
    writer_waker: &Mutex<Option<Waker>>,
    cx: &mut Context<'_>,
    buf: &[u8],
) -> Poll<Result<usize, io::Error>> {
    if closed.load(Ordering::SeqCst) {
        return Poll::Ready(Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "Relay pipe closed",
        )));
    }
    let mut bytes = buffer.lock().unwrap();
    if bytes.len() + buf.len() > PIPE_BUFFER {
        *writer_waker.lock().unwrap() = Some(cx.waker().clone());
        return Poll::Pending;
    }
    bytes.extend_from_slice(buf);
    if let Some(waker) = reader_waker.lock().unwrap().take() {
        waker.wake();
    }
    Poll::Ready(Ok(buf.len()))
}

/// Wakes a reader that is waiting for more data (the reader half).
fn wake_readers(reader_waker: &Mutex<Option<Waker>>) {
    if let Ok(mut waker) = reader_waker.lock() {
        if let Some(waker) = waker.take() {
            waker.wake();
        }
    }
}

/// Re-registers the current task as a backpressured writer if the buffer is full.
fn wake_backpressure(buffer: &Mutex<BytesMut>, writer_waker: &Mutex<Option<Waker>>, cx: &mut Context<'_>) {
    let bytes = buffer.lock().unwrap();
    if bytes.len() >= PIPE_BUFFER {
        drop(bytes);
        *writer_waker.lock().unwrap() = Some(cx.waker().clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_pipe_relays_bytes_both_directions() {
        let (mut pipe, inbound, mut outbound) = RelayPipe::new();

        inbound.push(b"ping");
        let mut from_peer = Vec::new();
        pipe.read_buf(&mut from_peer).await.unwrap();
        assert_eq!(from_peer, b"ping", "inbound bytes must reach the HTTP side");

        pipe.write_all(b"pong").await.unwrap();
        let mut to_peer = Vec::new();
        outbound.read_buf(&mut to_peer).await.unwrap();
        assert_eq!(to_peer, b"pong", "HTTP bytes must reach the outbound side");
    }

    #[tokio::test]
    async fn test_pipe_inbound_close_surfaces_eof_after_data() {
        let (mut pipe, inbound, _outbound) = RelayPipe::new();

        inbound.push(b"hello");
        let mut buf = Vec::new();
        pipe.read_buf(&mut buf).await.unwrap();
        assert_eq!(buf, b"hello");

        inbound.close();
        let mut eof = [0u8; 1];
        let n = pipe.read(&mut eof).await.unwrap();
        assert_eq!(n, 0, "closing the inbound side must surface EOF to a read");
    }

    #[test]
    fn test_pipe_write_after_shutdown_errors() {
        let (pipe, _inbound, _outbound) = RelayPipe::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut pipe = pipe;
            AsyncWriteExt::shutdown(&mut pipe).await.unwrap();
            assert!(pipe.write(b"late").await.is_err(), "writing after shutdown must fail");
        });
    }
}