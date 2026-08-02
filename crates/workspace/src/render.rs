use gpui::{
    App, Div, ParentElement, SharedString, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{ActiveTheme, list::ListItem};
use module::{AnyDynamic, Command};

pub(crate) const COMMAND_ITEM_HEIGHT: f32 = 48.0;
pub(crate) const DYNAMIC_ITEM_HEIGHT: f32 = 125.0;

pub fn render_command_item(
    id: u64,
    command: &Command,
    regex_query: Option<&str>,
    selected: bool,
    cx: &App,
) -> ListItem {
    let title: SharedString = regex_query
        .map(|query| format!("Open \"{query}\" with {}", command.title))
        .unwrap_or_else(|| command.title.clone())
        .into();
    let subtitle = command
        .subtitle
        .as_ref()
        .or(command.description.as_ref())
        .cloned();

    let mut content = div()
        .min_w_0()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(
            div()
                .w_full()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .child(title),
        );

    if let Some(subtitle) = subtitle {
        content = content.child(
            div()
                .w_full()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .child(subtitle),
        );
    }

    let row = div()
        .h(px(COMMAND_ITEM_HEIGHT))
        .min_w_0()
        .flex()
        .items_center()
        .gap_3()
        .child(command.icon.clone().background_color(cx.theme().secondary))
        .child(content)
        .when_some(command.shortcut.clone(), |this, shortcut| {
            this.child(
                div()
                    .flex_none()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(shortcut),
            )
        });

    ListItem::new(("workspace-command-item", id))
        .selected(selected)
        .px_2()
        .py_0()
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
