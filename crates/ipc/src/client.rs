use crate::{KeywayServiceClient, server::get_socket_path};
use keyway_net::async_net::UnixStream;
use tarpc::client;
use tarpc::context;
use tarpc::tokio_serde::formats::Json;
use tokio_util::codec::LengthDelimitedCodec;

async fn connect() -> anyhow::Result<KeywayServiceClient> {
    let socket_path = get_socket_path();
    let stream = UnixStream::connect(&socket_path).await?;

    let framed = tokio_util::codec::Framed::new(stream, LengthDelimitedCodec::new());
    let transport = tarpc::serde_transport::new(framed, Json::default());

    let client = KeywayServiceClient::new(client::Config::default(), transport).spawn();
    Ok(client)
}

pub fn client_connect() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let client = connect().await?;

        let commands = client.commands(context::current()).await?;

        // TODO 根据返回的commands生成命令

        Ok(())
    })
}
