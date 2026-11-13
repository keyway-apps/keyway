use gpui_kit::{
    App, AppContext, Context, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render, WeakEntity,
    Window, div,
};
use workspace::{
    Workspace,
    item::{Item, ItemEvent},
};

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        let item = cx.new(|cx| CommandPalette::new(workspace.weak_handle(), cx));
        workspace.add_item(Box::new(item), true, true, window, cx);
    })
    .detach();
}

pub struct CommandPalette {
    focus_handle: FocusHandle,
    workspace: WeakEntity<Workspace>,
}

impl CommandPalette {
    fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            workspace,
        }
    }
}

impl Item for CommandPalette {
    type Event = ItemEvent;

    fn on_removed(&mut self, _cx: &mut Context<Self>) {
        todo!()
    }
}

impl Render for CommandPalette {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Command Palette")
    }
}

impl EventEmitter<ItemEvent> for CommandPalette {}

impl Focusable for CommandPalette {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
