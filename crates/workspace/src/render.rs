use gpui::{
    App, Div, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled, Window, div, hsla, prelude::FluentBuilder, px, rgba,
};
use gpui_component::{ActiveTheme, Selectable, list::ListItem};
use module::{AnyDynamic, Command};

pub(crate) const COMMAND_ITEM_HEIGHT: f32 = 40.0;
pub(crate) const DYNAMIC_ITEM_HEIGHT: f32 = 125.0;

pub fn render_item(id: u64, command: &Command, selected: bool, window: &mut Window, cx: &mut App) -> ListItem {

    let icon = command.icon.clone().render(window, cx).into_any_element();

    let mut content = div()
        .w_full()
        .h_full()
        .flex_1()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .overflow_hidden()
        .child(icon)
        .child(
            div()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .child(SharedString::from(command.title.clone())),
        );

    if let Some(subtitle) = &command.subtitle {
        content = content.child(
            div()
                .flex_1()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .child(SharedString::from(subtitle.clone())),
        );
    }

    let bg_color = if selected {
        rgba(0xF5F5F5FF)
    } else {
        rgba(0x00000000)
    };

    let row = div()
        .h(px(COMMAND_ITEM_HEIGHT))
        .w_full()
        .px_2p5()
        .py_0()
        .m_0()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .bg(bg_color)
        .gap_2()
        .rounded_sm()
        .overflow_hidden()
        .child(content);

    ListItem::new(("workspace-command-item", id))
        .p_0()
        .m_0()
        .rounded(px(6.))
        .child(row)
}

pub fn render_dynamic_item(
    id: u64,
    dynamic: &AnyDynamic,
    selected: bool,
    window: &mut Window,
    cx: &mut App,
) -> ListItem {
    ListItem::new(("workspace-dynamic-item", id))
        .selected(selected)
        .h(px(DYNAMIC_ITEM_HEIGHT))
        .p_0()
        .overflow_hidden()
        .child(
            div()
                .size_full()
                .overflow_hidden()
                .child(dynamic.render(window, cx)),
        )
}

pub fn render_section_header(title: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .h(px(28.0))
        .px_2()
        .flex()
        .items_center()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(title.into())
}
