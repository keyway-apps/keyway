use anyhow::Result;
use gpui::App;
use gpui_tokio::Tokio;
use net::UnixListener;
use std::{path::PathBuf, sync::mpsc};
use tarpc::{server::BaseChannel, tokio_serde::formats::Json};
use tokio_util::codec::LengthDelimitedCodec;

pub fn get_socket_path() -> PathBuf {
    paths::temp_dir().join("keyway.sock")
}

pub fn start_server(cx: &App) {
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

            // let framed = tokio_util::codec::Framed::new(stream, LengthDelimitedCodec::new());
            // let transport = tarpc::serde_transport::new(framed, Json::default());

            // let channel = BaseChannel::with_defaults(transport);

            // tokio::spawn(channel.execute(server::ServerImpl::new(cx.clone()).serve()));
        }
    })
    .detach();
}
