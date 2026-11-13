use std::any::TypeId;

use gpui_kit::{
    AnyView, App, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled, WeakEntity, Window,
    WindowControlArea, base::h_flex, rgb,
};

use crate::{Workspace, item::ItemHandle};

pub trait TitleItemView: Render {
    fn set_active_item(&mut self, active_item: Option<&dyn ItemHandle>, window: &mut Window, cx: &mut Context<Self>);
    fn hide(&self, cx: &App) -> bool;
}

trait TitleItemViewHandle: Send {
    fn to_any(&self) -> AnyView;
    fn set_active_item(&self, active_item: Option<&dyn ItemHandle>, window: &mut Window, cx: &mut App);
    fn hide(&self, cx: &App) -> bool;
    fn item_type(&self) -> TypeId;
}

pub struct TitleBar {
    left_items: Vec<Box<dyn TitleItemViewHandle>>,
    right_items: Vec<Box<dyn TitleItemViewHandle>>,
    workspace: WeakEntity<Workspace>,
    active_item: Option<Box<dyn ItemHandle>>,
}

impl TitleBar {
    pub fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<TitleBar>) -> Self {
        Self {
            left_items: Default::default(),
            right_items: Default::default(),
            workspace,
            active_item: Default::default(),
        }
    }

    pub fn active_item(&self) -> Option<Box<dyn ItemHandle>> {
        self.active_item.clone()
    }

    pub fn add_left_item<T>(&mut self, item: Entity<T>, window: &mut Window, cx: &mut Context<Self>)
    where
        T: 'static + TitleItemView,
    {
        let active_item = self.active_item();
        item.set_active_item(active_item.as_deref(), window, cx);

        self.left_items.push(Box::new(item));
        cx.notify();
    }

    pub fn add_right_item<T>(&mut self, item: Entity<T>, window: &mut Window, cx: &mut Context<Self>)
    where
        T: 'static + TitleItemView,
    {
        let active_item = self.active_item();
        item.set_active_item(active_item.as_deref(), window, cx);

        self.right_items.push(Box::new(item));
        cx.notify();
    }

    pub fn set_active_item(&mut self, item: Option<Box<dyn ItemHandle>>, window: &mut Window, cx: &mut Context<Self>) {
        self.active_item = item;
        self.update_active_item(window, cx);
    }

    fn update_active_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active_item = self.active_item();
        for item in self.left_items.iter().chain(&self.right_items) {
            item.set_active_item(active_item.as_deref(), window, cx);
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .h_16()
            .gap_2()
            .px_4()
            .border_b_1()
            .border_color(rgb(0xE7E7E7))
            .window_control_area(WindowControlArea::Drag)
            .child(self.render_left_tools(cx))
            .child(self.render_right_tools(cx))
    }
}

impl TitleBar {
    fn render_left_tools(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().min_w_0().flex_1().gap_2().overflow_x_hidden().children(
            self.left_items
                .iter()
                .enumerate()
                .filter_map(|(_, item)| render_hideable_item(item.as_ref(), cx)),
        )
    }

    fn render_right_tools(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().gap_2().overflow_x_hidden().children(
            self.right_items
                .iter()
                .enumerate()
                .filter_map(|(_, item)| render_hideable_item(item.as_ref(), cx)),
        )
    }
}

fn render_hideable_item(item: &dyn TitleItemViewHandle, cx: &App) -> Option<impl IntoElement> {
    if item.hide(cx) {
        return None;
    }
    Some(item.to_any().into_any_element())
}

impl<T: TitleItemView> TitleItemViewHandle for Entity<T> {
    fn to_any(&self) -> AnyView {
        self.clone().into()
    }

    fn set_active_item(&self, active_item: Option<&dyn ItemHandle>, window: &mut Window, cx: &mut App) {
        self.update(cx, |this, cx| this.set_active_item(active_item, window, cx));
    }

    fn hide(&self, cx: &App) -> bool {
        self.read(cx).hide(cx)
    }

    fn item_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl From<&dyn TitleItemViewHandle> for AnyView {
    fn from(val: &dyn TitleItemViewHandle) -> Self {
        val.to_any()
    }
}
