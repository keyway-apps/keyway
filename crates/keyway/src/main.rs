use std::sync::Arc;

use gpui_kit::component::theme::{Theme, ThemeMode};
use gpui_kit::{
    App, AppContext, Application, Bounds, TaskExt, WindowBounds, WindowKind, WindowOptions, platform, px, size,
};

use assets::Assets;
use ipc::server::prepare_socket;
use workspace::{AppState, WorkspaceStore, open_new};

fn main() {
    ktracing::init();
    i18n::init();

    if let Err(_) = prepare_socket() {
        // TODO 通过ipc处理命令并直接返回
        return;
    }

    Application::with_platform(platform::current_platform(false))
        .with_assets(Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Light, None, cx);

            tray::init(cx);

            let workspace_store = cx.new(|_| WorkspaceStore::new());

            let app_state = Arc::new(AppState { workspace_store });
            AppState::set_global(app_state.clone(), cx);
            command::init(cx);

            command_palette::init(cx);

            create_workspace(app_state, cx);
        });
}

fn create_workspace(app_state: Arc<AppState>, cx: &mut App) {
    let display_id = cx.primary_display().map(|display| display.id());
    let size = size(px(750.), px(475.));
    let bounds = WindowBounds::Windowed(Bounds::centered(display_id, size.clone(), cx));

    let options = WindowOptions {
        focus: false,
        show: false,
        window_bounds: Some(bounds),
        titlebar: None,
        is_movable: true,
        is_resizable: false,
        kind: WindowKind::PopUp,
        display_id,
        window_min_size: Some(size),
        ..Default::default()
    };

    open_new(app_state, options, cx, |workspace, window, cx| {
        workspace.update_pinned_workspace(window, cx);
    })
    .detach_and_log_err(cx);
}
