use anyhow::Context as _;
use gpui_kit::{App, Global, Image, ImageFormat, MenuItem, actions};
use gpui_tray::{Icon, Tray};
use i18n::tr;

rust_i18n::i18n!("../../locales", fallback = "en");

pub fn init(cx: &mut App) {
    tracing::debug!("Initializing system tray");
    if let Err(error) = setup(cx) {
        tracing::error!(?error, "Failed to initialize system tray; exiting application");
        cx.quit();
    }
}

actions!(tray, [OpenKeyway, Documentation, About, CheckUpdates, Settings, Quit]);

struct GlobalTray(Tray);

impl Global for GlobalTray {}

fn setup(cx: &mut App) -> anyhow::Result<()> {
    let icon_bytes = cx
        .asset_source()
        .load("icons/logo.png")
        .context("failed to load tray icon asset `icons/logo.png`")?
        .context("tray icon asset `icons/logo.png` is unavailable")?;
    let image = Image::from_bytes(ImageFormat::Png, icon_bytes.into_owned());
    let icon = Icon::from_gpui(&image, cx)
        .build()
        .context("failed to build tray icon")?;

    let tray = Tray::builder()
        .icon(icon)
        .title("Keyway")
        .tooltip("Keyway")
        .menu(|_| build_menu())
        .on_activate_with(|_, cx| {
            tracing::debug!("System tray activated");
            workspace::active_pinned_window(cx);
        })
        .build(cx)
        .context("failed to create system tray")?;

    cx.on_action(|_: &OpenKeyway, cx| {
        tracing::debug!("Opening main window from tray menu");
        workspace::active_pinned_window(cx);
    });
    cx.on_action(quit);

    cx.set_global(GlobalTray(tray));
    tracing::info!("System tray initialized");

    Ok(())
}

fn build_menu() -> Vec<MenuItem> {
    vec![
        MenuItem::action(tr!("tray.open"), OpenKeyway),
        MenuItem::separator(),
        MenuItem::action(tr!("tray.documentation"), Documentation),
        MenuItem::action(tr!("tray.about"), About),
        MenuItem::action(tr!("tray.check_updates"), CheckUpdates),
        MenuItem::action(tr!("tray.settings"), Settings),
        MenuItem::separator(),
        MenuItem::action(tr!("tray.quit"), Quit),
    ]
}

fn quit(_: &Quit, cx: &mut App) {
    tracing::info!("Quit requested from system tray");
    let tray = cx.global::<GlobalTray>().0.clone();
    match tray.close(cx) {
        Ok(()) => tracing::debug!("System tray closed"),
        Err(error) => tracing::warn!(?error, "Failed to close system tray before quitting"),
    }
    cx.quit();
}
