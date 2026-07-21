use anyhow::Result;
use futures::prelude::*;
use gpui::App;
use gpui_tokio::Tokio;
use keyway_net::async_net::UnixListener;
use std::{path::PathBuf, sync::mpsc};
use tarpc::context::Context;
use tarpc::server::Channel;
use tarpc::{server::BaseChannel, tokio_serde::formats::Json};
use tokio_util::codec::LengthDelimitedCodec;

use crate::KeywayService;

pub struct IpcServerHandle {
    socket_path: PathBuf,
}

impl Drop for IpcServerHandle {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.socket_path) {
            // Don't warn if the file doesn't exist - that's expected if the socket
            // was never created or was already cleaned up
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("Failed to clean up IPC socket: {}", e);
            }
        }
    }
}

pub fn get_socket_path() -> PathBuf {
    keyway_paths::temp_dir().join("keyway.sock")
}

pub fn is_daemon_running() -> bool {
    let socket_path = get_socket_path();
    keyway_net::UnixStream::connect(&socket_path).is_ok()
}

#[derive(Clone)]
pub struct KeywayServer {}

impl KeywayServer {
    fn new() -> Self {
        Self {}
    }
}

impl KeywayService for KeywayServer {
    async fn execute(self, context: Context) -> () {
        todo!()
    }

    async fn commands(self, context: Context) -> () {
        todo!()
    }
}

pub fn prepare_socket() -> anyhow::Result<PathBuf> {
    let socket_path = get_socket_path();

    if socket_path.exists() {
        if is_daemon_running() {
            anyhow::bail!("Another instance is already running");
        }
        // Remove stale socket
        std::fs::remove_file(&socket_path)?;
    }

    Ok(socket_path)
}

pub fn start_server(cx: &App) -> anyhow::Result<IpcServerHandle> {
    let socket_path = get_socket_path();
    let socket_path_clone = socket_path.clone();

    let (bind_tx, bind_rx) = mpsc::channel::<Result<(), std::io::Error>>();

    Tokio::spawn(cx, async move {
        let listener = match UnixListener::bind(&socket_path_clone) {
            Ok(listener) => {
                let _ = bind_tx.send(Ok(()));
                listener
            }
            Err(e) => {
                let _ = bind_tx.send(Err(e));
                return;
            }
        };

        tracing::info!("IPC server listening on {:?}", socket_path_clone);

        loop {
            let (stream, _) = match listener.accept().await {
                Ok((stream, addr)) => (stream, addr),
                Err(e) => {
                    tracing::warn!("Failed to accept IPC connection: {}", e);
                    continue;
                }
            };

            let framed = tokio_util::codec::Framed::new(stream, LengthDelimitedCodec::new());
            let transport = tarpc::serde_transport::new(framed, Json::default());

            let server = KeywayServer::new();

            let channel = BaseChannel::with_defaults(transport);

            tokio::spawn(
                channel
                    .execute(server.serve())
                    .for_each(|response| async move {
                        tokio::spawn(response);
                    }),
            );
        }
    })
    .detach();

    match bind_rx.recv() {
        Ok(Ok(())) => Ok(IpcServerHandle { socket_path }),
        Ok(Err(e)) => anyhow::bail!("Failed to bind IPC socket: {}", e),
        Err(_) => anyhow::bail!("IPC server task terminated unexpectedly before binding socket"),
    }
}
