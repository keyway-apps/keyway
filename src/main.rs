use gpui::{
    App, AppContext, Application, TitlebarOptions, WindowBounds, WindowOptions, bounds, point, px,
    size,
};
use gpui_platform;
use kw_workspace::Workspace;

fn main() {
    kw_tracing::init();

    let app = Application::with_platform(gpui_platform::current_platform(false));

    app.run(move |cx| {
        kw_i18n::init("en");

        initialize_workspace(cx);
    });
}

fn initialize_workspace(cx: &mut App) {
    cx.open_window(
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("GPUI Typography".into()),
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(bounds(
                point(px(0.0), px(0.0)),
                size(px(920.), px(720.)),
            ))),
            ..Default::default()
        },
        |_window, cx| cx.new(|_cx| Workspace::new()),
    ).unwrap();
}
