use std::{rc::Rc, sync::Arc};

use gpui_kit::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Pixels, Render, Role, SharedString, Size, StatefulInteractiveElement, Styled, Task,
    Window,
    base::{VirtualListScrollHandle, v_flex, v_virtual_list},
    component::scroll::Scrollbar,
    div,
    prelude::FluentBuilder,
};
use i18n::tr;
use workspace::{
    Workspace,
    item::{Item, ItemEvent},
    searchable::{SearchOptions, SearchQuery, SearchableItem, SearchableItemHandle},
};

rust_i18n::i18n!("../../locales", fallback = "en");

pub fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let item = cx.new(CommandPalette::new);
    workspace.add_item(Box::new(item), true, true, window, cx);
}

#[derive(Clone, PartialEq)]
enum CommandRow {
    Heading(SharedString),
    Item(usize),
}

pub struct CommandPalette {
    focus_handle: FocusHandle,
    scroll_handle: VirtualListScrollHandle,
    rows: Vec<CommandRow>,
    row_sizes: Rc<Vec<Size<Pixels>>>,
}

impl CommandPalette {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            scroll_handle: VirtualListScrollHandle::new(),
            rows: Vec::new(),
            row_sizes: Rc::new(Vec::new()),
        }
    }

    fn render_visible_row(&mut self, row_ix: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        div().into_any_element()
    }

    fn render_empty(&self) -> AnyElement {
        let message = tr!("command_palette.empty");

        div()
            .py_6()
            .w_full()
            .text_center()
            .text_sm()
            .child(message)
            .into_any_element()
    }
}

impl Item for CommandPalette {
    type Event = ItemEvent;

    fn as_searchable(&self, handle: &Entity<Self>, _: &App) -> Option<Box<dyn SearchableItemHandle>> {
        Some(Box::new(handle.clone()))
    }

    fn on_removed(&mut self, _cx: &mut Context<Self>) {
        todo!()
    }
}

impl SearchableItem for CommandPalette {
    fn supported_options(&self) -> SearchOptions {
        SearchOptions {
            case: true,
            word: true,
            regex: true,
        }
    }

    fn find_matches(&mut self, _query: Arc<SearchQuery>, _window: &mut Window, _cx: &mut Context<Self>) -> Task<()> {
        todo!()
    }
}

impl Render for CommandPalette {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows_count = self.rows.len();
        let row_sizes = self.row_sizes.clone();

        v_flex()
            .id("command-palette")
            .role(Role::ListBox)
            .size_full()
            .track_focus(&self.focus_handle)
            .overflow_hidden()
            .px_2p5()
            .when(rows_count == 0, |this| this.child(self.render_empty()))
            .when(rows_count > 0, |this| {
                this.child(
                    v_virtual_list(
                        cx.entity(),
                        "command-list",
                        row_sizes,
                        move |this, visible_range, window, cx| {
                            visible_range
                                .map(|row_ix| this.render_visible_row(row_ix, window, cx))
                                .collect::<Vec<_>>()
                        },
                    )
                    .track_scroll(&self.scroll_handle),
                )
                .child(Scrollbar::vertical(&self.scroll_handle))
            })
    }
}

impl EventEmitter<ItemEvent> for CommandPalette {}

impl Focusable for CommandPalette {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
