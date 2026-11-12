use gpui_kit::{App, AppContext, Context, IntoElement, Render, Styled, WeakEntity, Window, div, px, rgb};
use i18n::tr;
use workspace::Workspace;

rust_i18n::i18n!("../../locales", fallback = "en");

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        let item = cx.new(|cx| ActionsBar::new(workspace, window, cx));
        workspace.set_titlebar_item(item.into(), cx);
    })
    .detach();
}

pub struct ActionsBar {
    workspace: WeakEntity<Workspace>,
}

impl ActionsBar {
    pub fn new(workspace: &Workspace, window: &mut Window, cx: &mut Context<ActionsBar>) -> Self {
        Self {
            workspace: workspace.weak_handle(),
        }
    }
}

impl Render for ActionsBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(40.))
            .border_t_1()
            .border_color(rgb(0xE7E7E7))
            .flex()
            .justify_between()
            .px_4()
    }
}
