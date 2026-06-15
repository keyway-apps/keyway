use std::{path::PathBuf, sync::OnceLock};

pub const APP_NAME: &str = "Keyway";

pub const APP_NAME_LOWERCASE: &str = {
    assert!(!APP_NAME.is_empty(), "APP_NAME must not be empty");
    assert!(APP_NAME.as_bytes().is_ascii(), "APP_NAME must be ASCII");
    const BYTES: [u8; APP_NAME.len()] = {
        let mut bytes = [0u8; APP_NAME.len()];
        let mut i = 0;
        while i < APP_NAME.len() {
            assert!(
                APP_NAME.as_bytes()[i] != b'/' && APP_NAME.as_bytes()[i] != b'\\',
                "APP_NAME must not contain path separators",
            );
            assert!(
                APP_NAME.as_bytes()[i] >= 0x20,
                "APP_NAME must not contain control characters"
            );
            bytes[i] = APP_NAME.as_bytes()[i];
            i += 1;
        }
        bytes.make_ascii_lowercase();
        bytes
    };
    match std::str::from_utf8(&BYTES) {
        Ok(s) => s,
        Err(_) => unreachable!(),
    }
};

static CURRENT_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn home_dir() -> &'static PathBuf {
    static HOME_DIR: OnceLock<PathBuf> = OnceLock::new();
    HOME_DIR.get_or_init(|| {
        log::debug!("paths: initializing home directory");
        dirs::home_dir().expect("Failed to get home dir")
    })
}

pub fn config_dir() -> &'static PathBuf {
    CONFIG_DIR.get_or_init(|| {
        if cfg!(target_os = "windows") {
            dirs::config_dir()
                .expect("failed to determine RoamingAppData directory")
                .join(APP_NAME)
        } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            if let Ok(flatpak_xdg_config) = std::env::var("FLATPAK_XDG_CONFIG_HOME") {
                flatpak_xdg_config.into()
            } else {
                dirs::config_dir().expect("failed to determine XDG_CONFIG_HOME directory")
            }
            .join(APP_NAME_LOWERCASE)
        } else {
            home_dir().join(".config").join(APP_NAME_LOWERCASE)
        }
    })
}

pub fn data_dir() -> &'static PathBuf {
    CURRENT_DATA_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            home_dir()
                .join("Library/Application Support")
                .join(APP_NAME)
        } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            if let Ok(flatpak_xdg_data) = std::env::var("FLATPAK_XDG_DATA_HOME") {
                flatpak_xdg_data.into()
            } else {
                dirs::data_local_dir().expect("failed to determine XDG_DATA_HOME directory")
            }
            .join(APP_NAME_LOWERCASE)
        } else if cfg!(target_os = "windows") {
            dirs::data_local_dir()
                .expect("failed to determine LocalAppData directory")
                .join(APP_NAME)
        } else {
            config_dir().clone() // Fallback
        }
    })
}

pub fn temp_dir() -> &'static PathBuf {
    static TEMP_DIR: OnceLock<PathBuf> = OnceLock::new();
    TEMP_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            dirs::cache_dir()
                .expect("failed to determine cachesDirectory directory")
                .join(APP_NAME)
        } else if cfg!(target_os = "windows") {
            dirs::cache_dir()
                .expect("failed to determine LocalAppData directory")
                .join(APP_NAME)
        } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            if let Ok(flatpak_xdg_cache) = std::env::var("FLATPAK_XDG_CACHE_HOME") {
                flatpak_xdg_cache.into()
            } else {
                dirs::cache_dir().expect("failed to determine XDG_CACHE_HOME directory")
            }
            .join(APP_NAME_LOWERCASE)
        } else {
            home_dir().join(".cache").join(APP_NAME_LOWERCASE)
        }
    })
}

pub fn logs_dir() -> &'static PathBuf {
    static LOGS_DIR: OnceLock<PathBuf> = OnceLock::new();
    LOGS_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            home_dir().join("Library/Logs").join(APP_NAME)
        } else {
            data_dir().join("logs")
        }
    })
}

pub fn extensions_dir() -> &'static PathBuf {
    static EXTENSIONS_DIR: OnceLock<PathBuf> = OnceLock::new();
    EXTENSIONS_DIR.get_or_init(|| home_dir().join(".keyway").join("extensions"))
}
