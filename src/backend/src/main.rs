// chorgly-backend: WebSocket server

mod state;
mod session;
mod persist;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use anyhow::Result;

use state::SharedState;

#[tokio::main]
async fn main() -> Result<()> {
  // Bind on all interfaces, IPv6 first (dual-stack on Linux serves v4 too).
  let addr: SocketAddr = "[::]:8765".parse()?;
  let listener = TcpListener::bind(addr).await?;
  eprintln!("chorgly-backend listening on {addr}");

  // Load DB from disk (or start fresh) and wrap in shared state.
  let state = Arc::new(SharedState::load_or_default("data").await?);

  // Spawn the hourly persistence task.
  {
    let s = Arc::clone(&state);
    tokio::spawn(async move {
      persist::flush_loop(s).await;
    });
  }

  // Accept WebSocket connections.
  loop {
    let (stream, peer) = listener.accept().await?;
    let s = Arc::clone(&state);
    tokio::spawn(async move {
      match accept_async(stream).await {
        Ok(ws) => session::run(ws, peer, s).await,
        Err(e) => eprintln!("WS handshake failed from {peer}: {e}"),
      }
    });
  }
}
