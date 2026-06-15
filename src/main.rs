use gpui::Application;
use gpui_platform;

fn main() {
    kw_tracing::init();

    let app = Application::with_platform(gpui_platform::current_platform(false));

    app.run(move |_cx| {
        kw_i18n::init("en");
    });
}
