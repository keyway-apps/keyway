use std::any::TypeId;

use gpui_kit::{
    AnyView, App, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled, Subscription,
    WeakEntity, Window, WindowControlArea,
    base::h_flex,
    component::{IconName, Sizable, button::Button},
    div,
    prelude::FluentBuilder,
    rgb,
};

use crate::{Workspace, item::ItemHandle, pane::Pane};

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
    pane: Entity<Pane>,
    workspace: WeakEntity<Workspace>,
    _observe_pane: Subscription,
}

impl TitleBar {
    pub fn new(
        pane: &Entity<Pane>,
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<TitleBar>,
    ) -> Self {
        let mut this = Self {
            left_items: Default::default(),
            right_items: Default::default(),
            pane: pane.clone(),
            workspace,
            _observe_pane: cx.observe_in(pane, window, |this, _, window, cx| {
                this.update_active_item(window, cx);
            }),
        };
        this.update_active_item(window, cx);
        this
    }

    pub fn add_left_item<T>(&mut self, item: Entity<T>, window: &mut Window, cx: &mut Context<Self>)
    where
        T: 'static + TitleItemView,
    {
        let active_item = self.pane.read(cx).active_item(cx);
        item.set_active_item(active_item.as_deref(), window, cx);

        self.left_items.push(Box::new(item));
        cx.notify();
    }

    pub fn add_right_item<T>(&mut self, item: Entity<T>, window: &mut Window, cx: &mut Context<Self>)
    where
        T: 'static + TitleItemView,
    {
        let active_item = self.pane.read(cx).active_item(cx);
        item.set_active_item(active_item.as_deref(), window, cx);

        self.right_items.push(Box::new(item));
        cx.notify();
    }

    pub fn item_of_type<T: TitleItemView>(&self) -> Option<Entity<T>> {
        self.left_items
            .iter()
            .chain(self.right_items.iter())
            .find_map(|item| item.to_any().downcast().ok())
    }

    pub fn position_of_item<T>(&self) -> Option<usize>
    where
        T: TitleItemView,
    {
        for (index, item) in self.left_items.iter().enumerate() {
            if item.item_type() == TypeId::of::<T>() {
                return Some(index);
            }
        }
        for (index, item) in self.right_items.iter().enumerate() {
            if item.item_type() == TypeId::of::<T>() {
                return Some(index + self.left_items.len());
            }
        }
        None
    }

    fn update_active_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active_item = self.pane.read(cx).active_item(cx);
        for item in self.left_items.iter().chain(&self.right_items) {
            item.set_active_item(active_item.as_deref(), window, cx);
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let can_go_back = self.pane.read(cx).can_go_back(cx);

        h_flex()
            .h_16()
            .gap_2()
            .px_4()
            .border_b_1()
            .border_color(rgb(0xE7E7E7))
            .window_control_area(WindowControlArea::Drag)
            .when(can_go_back, |bar| {
                bar.child(
                    div().occlude().child(
                        Button::new("btn-back")
                            .small()
                            .outline()
                            .icon(IconName::ArrowLeft)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.workspace
                                    .update(cx, |workspace, cx| {
                                        workspace.go_back(window, cx);
                                    })
                                    .ok();
                            })),
                    ),
                )
            })
            .child(self.render_left_tools(cx))
            .child(self.render_right_tools(cx))
    }
}

impl TitleBar {
    fn render_left_tools(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().min_w_0().flex_1().gap_2().overflow_x_hidden().children(
            self.left_items
                .iter()
                .filter_map(|item| render_hideable_item(item.as_ref(), cx)),
        )
    }

    fn render_right_tools(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().gap_2().overflow_x_hidden().children(
            self.right_items
                .iter()
                .filter_map(|item| render_hideable_item(item.as_ref(), cx)),
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
