use std::{
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use net::{UnixListener, UnixStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::timeout,
};

static SOCKET_ID: AtomicUsize = AtomicUsize::new(0);

#[tokio::test]
async fn listener_connect_roundtrip() -> io::Result<()> {
    with_timeout(async {
        let socket = SocketPath::new("roundtrip");
        let listener = UnixListener::bind(socket.path())?;

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await?;
            let mut request = [0; 4];
            stream.read_exact(&mut request).await?;
            assert_eq!(&request, b"ping");

            stream.write_all(b"pong").await?;
            stream.shutdown().await
        });

        let mut client = UnixStream::connect(socket.path()).await?;
        client.write_all(b"ping").await?;

        let mut response = [0; 4];
        client.read_exact(&mut response).await?;
        assert_eq!(&response, b"pong");

        server.await.map_err(join_error)??;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn stream_pair_is_bidirectional() -> io::Result<()> {
    with_timeout(async {
        let (mut left, mut right) = UnixStream::pair()?;

        let left_task = tokio::spawn(async move {
            left.write_all(b"left-to-right").await?;

            let mut response = [0; 13];
            left.read_exact(&mut response).await?;
            assert_eq!(&response, b"right-to-left");
            Ok::<_, io::Error>(())
        });

        let right_task = tokio::spawn(async move {
            let mut request = [0; 13];
            right.read_exact(&mut request).await?;
            assert_eq!(&request, b"left-to-right");

            right.write_all(b"right-to-left").await
        });

        left_task.await.map_err(join_error)??;
        right_task.await.map_err(join_error)??;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn owned_halves_match_tokio_split_behavior() -> io::Result<()> {
    with_timeout(async {
        let socket = SocketPath::new("split");
        let listener = UnixListener::bind(socket.path())?;

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await?;
            let (mut reader, mut writer) = stream.into_split();

            let mut request = [0; 5];
            reader.read_exact(&mut request).await?;
            assert_eq!(&request, b"split");

            writer.write_all(b"reply").await?;
            writer.shutdown().await
        });

        let stream = UnixStream::connect(socket.path()).await?;
        let (mut reader, mut writer) = stream.into_split();

        writer.write_all(b"split").await?;
        writer.shutdown().await?;

        let mut response = [0; 5];
        reader.read_exact(&mut response).await?;
        assert_eq!(&response, b"reply");

        server.await.map_err(join_error)??;
        Ok(())
    })
    .await
}

#[cfg(target_os = "windows")]
#[tokio::test]
async fn windows_listener_accepts_plain_uds_windows_client() -> io::Result<()> {
    use std::io::{Read as _, Write as _};

    with_timeout(async {
        let socket = SocketPath::new("raw-client");
        let listener = UnixListener::bind(socket.path())?;

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await?;
            let mut request = [0; 3];
            stream.read_exact(&mut request).await?;
            assert_eq!(&request, b"raw");

            stream.write_all(b"tok").await?;
            stream.shutdown().await
        });

        let path = socket.path().to_owned();
        let client = tokio::task::spawn_blocking(move || {
            let mut stream = uds_windows::UnixStream::connect(path)?;
            stream.write_all(b"raw")?;

            let mut response = [0; 3];
            stream.read_exact(&mut response)?;
            assert_eq!(&response, b"tok");
            Ok::<_, io::Error>(())
        });

        client.await.map_err(join_error)??;
        server.await.map_err(join_error)??;
        Ok(())
    })
    .await
}

#[cfg(target_os = "windows")]
#[tokio::test]
async fn windows_stream_connects_to_plain_uds_windows_listener() -> io::Result<()> {
    use std::io::{Read as _, Write as _};

    with_timeout(async {
        let socket = SocketPath::new("raw-listener");
        let listener = uds_windows::UnixListener::bind(socket.path())?;

        let server = tokio::task::spawn_blocking(move || {
            let (mut stream, _) = listener.accept()?;
            let mut request = [0; 5];
            stream.read_exact(&mut request)?;
            assert_eq!(&request, b"async");

            stream.write_all(b"plain")?;
            Ok::<_, io::Error>(())
        });

        let mut client = UnixStream::connect(socket.path()).await?;
        client.write_all(b"async").await?;

        let mut response = [0; 5];
        client.read_exact(&mut response).await?;
        assert_eq!(&response, b"plain");

        server.await.map_err(join_error)??;
        Ok(())
    })
    .await
}

async fn with_timeout(future: impl std::future::Future<Output = io::Result<()>>) -> io::Result<()> {
    timeout(Duration::from_secs(5), future)
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Unix socket test timed out"))?
}

fn join_error(error: tokio::task::JoinError) -> io::Error {
    io::Error::other(error)
}

struct SocketPath(PathBuf);

impl SocketPath {
    fn new(test_name: &str) -> Self {
        let id = SOCKET_ID.fetch_add(1, Ordering::Relaxed);
        let mut path = std::env::temp_dir();
        path.push(format!(
            "kw-net-{}-{id}-{test_name}.sock",
            std::process::id()
        ));

        if cfg!(target_os = "windows") {
            assert!(
                path.to_string_lossy().len() < 100,
                "Windows AF_UNIX paths must be shorter than sockaddr_un::sun_path: {}",
                path.display()
            );
        }

        let _ = std::fs::remove_file(&path);
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for SocketPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
