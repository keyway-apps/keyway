#[cfg(not(target_os = "windows"))]
pub use tokio::net::{UnixListener, UnixStream};

#[cfg(target_os = "windows")]
pub use windows::{UnixListener, UnixStream};

#[cfg(target_os = "windows")]
pub mod windows {
    use std::{
        fmt, io,
        os::windows::io::{AsRawSocket, AsSocket, BorrowedSocket, FromRawSocket, IntoRawSocket},
        path::Path,
        pin::Pin,
        task::{Context, Poll},
    };

    use tokio::{
        io::{AsyncRead, AsyncWrite, ReadBuf},
        net::TcpStream,
        task::{self, JoinError},
    };

    pub type OwnedReadHalf = tokio::net::tcp::OwnedReadHalf;
    pub type OwnedWriteHalf = tokio::net::tcp::OwnedWriteHalf;

    pub struct UnixListener(uds_windows::UnixListener);

    impl UnixListener {
        pub fn bind<P: AsRef<Path>>(path: P) -> io::Result<Self> {
            uds_windows::UnixListener::bind(path).map(Self)
        }

        pub async fn accept(&self) -> io::Result<(UnixStream, uds_windows::SocketAddr)> {
            let listener = self.try_clone()?;
            task::spawn_blocking(move || listener.0.accept())
                .await
                .map_err(join_error)?
                .and_then(|(stream, addr)| {
                    UnixStream::from_uds(stream).map(|stream| (stream, addr))
                })
        }

        fn try_clone(&self) -> io::Result<Self> {
            self.0.try_clone().map(Self)
        }

        pub fn local_addr(&self) -> io::Result<uds_windows::SocketAddr> {
            self.0.local_addr()
        }

        pub fn take_error(&self) -> io::Result<Option<io::Error>> {
            self.0.take_error()
        }
    }

    impl fmt::Debug for UnixListener {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    impl AsRawSocket for UnixListener {
        fn as_raw_socket(&self) -> std::os::windows::io::RawSocket {
            self.0.as_raw_socket()
        }
    }

    impl AsSocket for UnixListener {
        fn as_socket(&self) -> BorrowedSocket<'_> {
            unsafe { BorrowedSocket::borrow_raw(self.as_raw_socket()) }
        }
    }

    pub struct UnixStream {
        inner: TcpStream,
    }

    impl UnixStream {
        pub async fn connect<P: AsRef<Path>>(path: P) -> io::Result<Self> {
            let path = path.as_ref().to_owned();
            let stream = task::spawn_blocking(move || uds_windows::UnixStream::connect(path))
                .await
                .map_err(join_error)??;

            Self::from_uds(stream)
        }

        pub fn pair() -> io::Result<(Self, Self)> {
            let (left, right) = uds_windows::UnixStream::pair()?;
            Ok((Self::from_uds(left)?, Self::from_uds(right)?))
        }

        pub fn into_split(self) -> (OwnedReadHalf, OwnedWriteHalf) {
            self.inner.into_split()
        }

        fn from_uds(stream: uds_windows::UnixStream) -> io::Result<Self> {
            stream.set_nonblocking(true)?;
            let raw = stream.into_raw_socket();
            let stream = unsafe { std::net::TcpStream::from_raw_socket(raw) };
            TcpStream::from_std(stream).map(|inner| Self { inner })
        }
    }

    impl fmt::Debug for UnixStream {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.inner.fmt(f)
        }
    }

    impl AsyncRead for UnixStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Pin::new(&mut self.inner).poll_read(cx, buf)
        }
    }

    impl AsyncWrite for UnixStream {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            Pin::new(&mut self.inner).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Pin::new(&mut self.inner).poll_flush(cx)
        }

        fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Pin::new(&mut self.inner).poll_shutdown(cx)
        }
    }

    impl AsRawSocket for UnixStream {
        fn as_raw_socket(&self) -> std::os::windows::io::RawSocket {
            self.inner.as_raw_socket()
        }
    }

    impl AsSocket for UnixStream {
        fn as_socket(&self) -> BorrowedSocket<'_> {
            self.inner.as_socket()
        }
    }

    fn join_error(error: JoinError) -> io::Error {
        io::Error::other(error)
    }
}
