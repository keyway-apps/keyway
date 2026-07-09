#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use self::windows::{UnixListener, UnixStream};
#[cfg(not(target_os = "windows"))]
pub use tokio::net::{UnixListener, UnixStream};
