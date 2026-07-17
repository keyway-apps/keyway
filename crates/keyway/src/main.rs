use gpui::Application;
use gpui_platform;
use gpui_component::theme::{Theme, ThemeMode};

use assets::Assets;
use ipc::server::prepare_socket;

fn main() {
    ktracing::init();

    if let Err(_) = prepare_socket() {
        // TODO 通过ipc处理命令并直接返回
        return;
    }

    let app = Application::with_platform(gpui_platform::current_platform(false));

    app.with_assets(Assets).run(move |cx| {
        i18n::init("en");
        gpui_component::init(cx);
        Theme::change(ThemeMode::Light, None, cx);

        gpui_tokio::init(cx);

        command::init(cx);

        clipboard::init(cx);

        workspace::init(cx);
    });
}
