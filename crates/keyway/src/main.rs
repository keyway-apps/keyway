use gpui::Application;
use gpui_component::theme::{Theme, ThemeMode};
use gpui_platform;

use keyway_assets::Assets;
use keyway_ipc::server::prepare_socket;

fn main() {
    keyway_ktracing::init();

    if let Err(_) = prepare_socket() {
        // TODO 通过ipc处理命令并直接返回
        return;
    }

    let app = Application::with_platform(gpui_platform::current_platform(false));

    app.with_assets(Assets).run(move |cx| {
        keyway_i18n::init("en");
        gpui_component::init(cx);
        Theme::change(ThemeMode::Light, None, cx);

        gpui_tokio::init(cx);

        keyway_core::init(cx);

        keyway_clipboard::init(cx);

        keyway_workspace::init(cx);
    });
}
