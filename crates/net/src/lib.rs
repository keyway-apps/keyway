pub mod async_net;

#[cfg(not(target_os = "windows"))]
pub use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(target_os = "windows")]
pub use uds_windows::{UnixListener, UnixStream};
