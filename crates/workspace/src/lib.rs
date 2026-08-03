use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, IntoElement, Render, Window,
    WindowBounds, WindowKind, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::list::{List, ListState};
use gpui_component::{Root, Sizable};

use module::{Command, ModuleStore};

use crate::delegates::CommandListDelegate;
use crate::state::ViewMode;

mod delegates;
mod dynamic;
mod filter;
mod render;
mod section;
mod state;

pub static WIDTH: f32 = 750.0;
pub static HEIGHT: f32 = 475.0;

pub fn init(cx: &mut App) {
    let display_id = cx.primary_display().map(|display| display.id());

    let size = size(px(WIDTH), px(HEIGHT));
    let options = WindowOptions {
        focus: true,
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            display_id, size, cx,
        ))),
        titlebar: None,
        is_movable: false,
        kind: WindowKind::PopUp,
        display_id,
        ..Default::default()
    };

    cx.open_window(options, |window, cx| {
        cx.new(|cx| Root::new(cx.new(|cx| Workspace::new(window, cx)), window, cx))
    })
    .unwrap();
}

pub struct Workspace {
    pub(crate) view_mode: ViewMode,
    pub(crate) input_state: Entity<InputState>,
    pub(crate) command_list_state: Entity<ListState<CommandListDelegate>>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let module_context = ModuleStore::global(cx).read(cx).context();

        let commands: Vec<Command> = module_context
            .read(cx)
            .visible_commands()
            .cloned()
            .collect();

        let dynamics = module_context.read(cx).dynamics().cloned().collect();

        let delegate = CommandListDelegate::new(commands, dynamics);

        let command_list_state = cx.new(|cx| ListState::new(delegate, window, cx));
        let initial_selection = command_list_state.read(cx).delegate().selected_index();
        command_list_state.update(cx, |list, cx| {
            list.set_selected_index(initial_selection, window, cx);
        });

        let list_for_context = command_list_state.clone();
        cx.observe_in(
            &module_context,
            window,
            move |_this, module_context, window, cx| {
                let module_context = module_context.read(cx);
                let commands = module_context
                    .visible_commands()
                    .cloned()
                    .collect::<Vec<_>>();
                let dynamics = module_context.dynamics().cloned().collect::<Vec<_>>();
                list_for_context.update(cx, |list, cx| {
                    let selected = {
                        let delegate = list.delegate_mut();
                        delegate.update(commands, dynamics, cx);
                        delegate.selected_index()
                    };
                    list.set_selected_index(selected, window, cx);
                    cx.notify();
                });
            },
        )
        .detach();

        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search for apps and commands..."));

        let list_for_input = command_list_state.clone();
        cx.subscribe_in(
            &input_state,
            window,
            move |_this,
                  input: &Entity<InputState>,
                  event: &InputEvent,
                  window,
                  cx: &mut Context<Self>| {
                if let InputEvent::Change = event {
                    let text = input.read(cx).value().to_string();
                    list_for_input.update(cx, |list, cx| {
                        let selected = {
                            let delegate = list.delegate_mut();
                            delegate.set_query(text, cx);
                            delegate.selected_index()
                        };
                        list.set_selected_index(selected, window, cx);
                        cx.notify();
                    });
                }
            },
        )
        .detach();

        Self {
            view_mode: Default::default(),
            input_state,
            command_list_state,
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
            .child(div().flex_1().w_full().child(content))
            .child(
                div()
                    .w_full()
                    .px_2()
                    .border_t_1()
                    .border_color(rgb(0xCCCCCC)),
            )
    }
}

impl Workspace {
    fn render_content(&mut self) -> AnyElement {
        match self.view_mode {
            // The intermediate container needs an explicit size. Without it,
            // the List's `size_full` has no definite height to resolve against
            // and the virtual list can render as a zero-height child.
            ViewMode::Main => div()
                .size_full()
                .overflow_hidden()
                .p_2()
                .child(List::new(&self.command_list_state).size_full())
                .into_any_element(),
            ViewMode::View => div().child("view").into_any_element(),
        }
    }
}
