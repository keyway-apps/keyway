use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, IntoElement, Render, Window,
    WindowBounds, WindowKind, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::{Root, Sizable};

use crate::state::ViewMode;

mod delegates;
mod state;

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
    pub(crate) view_mode: ViewMode,
    pub(crate) input_state: Entity<InputState>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search for apps and commands..."));

        cx.subscribe(
            &input_state,
            move |_this, input: Entity<InputState>, event: &InputEvent, cx: &mut Context<Self>| {
                if let InputEvent::Change = event {
                    let text = input.read(cx).value().to_string();
                    tracing::info!("Search changed: {}", text);
                }
            },
        ).detach();

        Self {
            view_mode: Default::default(),
            input_state,
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.render_content();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xFFFFFF))
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .p_2()
                    .border_b_1()
                    .border_color(rgb(0xCCCCCC))
                    .child(
                        gpui_component::input::Input::new(&self.input_state)
                            .large()
                            .appearance(false)
                            .cleanable(true),
                    ),
            )
            .child(div().flex_1().size_full().px_2().child(content))
            .child(
                div()
                    .w_full()
                    .px_2()
                    .py_2()
                    .border_t_1()
                    .border_color(rgb(0xCCCCCC)),
            )
    }
}

impl Workspace {
    fn render_content(&mut self) -> AnyElement {
        match self.view_mode {
            ViewMode::Main => div().child("main").into_any_element(),
            ViewMode::View => div().child("view").into_any_element(),
        }
    }
}
