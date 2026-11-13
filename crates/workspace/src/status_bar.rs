use gpui_kit::{Context, IntoElement, Render, Styled, WeakEntity, Window, div, px, rgb};

use crate::{Workspace, item::ItemHandle};

pub trait StatusItemView: Render {
    fn set_active_item(&mut self, active_item: Option<&dyn ItemHandle>, window: &mut Window, cx: &mut Context<Self>);
}

pub struct StatusBar {
    workspace: WeakEntity<Workspace>,
    active_item: Option<Box<dyn ItemHandle>>,
}

impl StatusBar {
    pub fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<StatusBar>) -> Self {
        Self {
            workspace,
            active_item: Default::default(),
        }
    }

    pub fn set_active_item(&mut self, item: Option<Box<dyn ItemHandle>>, window: &mut Window, cx: &mut Context<Self>) {
        self.active_item = item;
    }
}

impl Render for StatusBar {
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
