use gpui::{
    App, AppContext, Bounds, Context, Entity, IntoElement, Render, Window, WindowBounds,
    WindowKind, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_component::Root;
use gpui_component::input::InputState;

pub static WIDTH: f32 = 750.0;
pub static HEIGHT: f32 = 475.0;

pub fn init(cx: &mut App) {
    let display_id = cx.displays().first().map(|d| d.id());

    let mut options = WindowOptions::default();

    options.focus = true;

    let size = size(px(WIDTH), px(HEIGHT));
    options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
        display_id, size, cx,
    )));

    options.titlebar = None;
    options.is_movable = false;
    options.kind = WindowKind::PopUp;

    cx.open_window(options, |window, cx| {
        cx.new(|cx| Root::new(cx.new(|cx| Workspace::new(window, cx)), window, cx))
    })
    .unwrap();
}

pub struct Workspace {
    pub(crate) input_state: Entity<InputState>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search for apps and commands..."));

        Self { input_state }
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
            .child(
                div()
                    .w_full()
                    .px_2()
                    .py_3()
                    .border_b_1()
                    .border_color(rgb(0xCCCCCC))
                    .child(
                        gpui_component::input::Input::new(&self.input_state)
                            .appearance(false)
                            .cleanable(true),
                    ),
            )
    }
}
