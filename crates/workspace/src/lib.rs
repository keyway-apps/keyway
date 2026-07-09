use gpui::{
    App, AppContext, Bounds, Context, IntoElement, Render, Window, WindowBounds, WindowKind, WindowOptions, div, prelude::*, px, rgb, size,
};

pub static WIDTH: f32 = 750.0;
pub static HEIGHT: f32 = 475.0;

pub fn init(cx: &mut App) {
    let display_id = cx
        .displays()
        .first()
        .map(|d| d.id());

    let mut options = WindowOptions::default();

    options.focus = true;
    
    let size = size(px(WIDTH), px(HEIGHT));
    options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(display_id, size, cx)));

    options.titlebar = None;
    options.is_movable = false;
    options.kind = WindowKind::PopUp;

    cx.open_window(options, |_window, cx| cx.new(|_cx| Workspace::new()))
        .unwrap();
}

pub struct Workspace {}

impl Workspace {
    pub fn new() -> Self {
        Self {}
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xFFFFFF))
            .overflow_hidden()
    }
}
