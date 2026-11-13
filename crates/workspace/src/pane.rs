use std::{num::NonZeroUsize, time::Duration};

use gpui_kit::{
    App, AppContext, Context, Entity, EntityId, FocusHandle, Focusable, IntoElement, ParentElement, Render, Styled,
    WeakEntity, Window,
    base::{PresencePhase, Transition},
    component::empty::{Empty, EmptyHeader, EmptyTitle},
    div,
    prelude::FluentBuilder,
    relative,
};
use ui::{NavMotion, NavOperation, NavStack, NavStackState};

use crate::{Workspace, item::ItemHandle};

pub struct Pane {
    // Cache only: oldest opened items come first; navigation does not reorder them.
    items: Vec<Box<dyn ItemHandle>>,
    focus_handle: FocusHandle,
    pub(crate) workspace: WeakEntity<Workspace>,
    // Includes the active item; None leaves the cache unbounded.
    max_items: Option<NonZeroUsize>,
    stack: Entity<NavStackState>,
}

impl Pane {
    pub fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        let stack = cx.new(|_cx| NavStackState::new());
        cx.observe(&stack, |_, _, cx| cx.notify()).detach();

        Self {
            items: Vec::new(),
            stack,
            workspace,
            max_items: NonZeroUsize::new(10),
            focus_handle: cx.focus_handle(),
        }
    }

    pub(crate) fn stack(&self) -> &Entity<NavStackState> {
        &self.stack
    }

    /// Reuses existing items and evicts the oldest inactive items when the cache is full.
    /// Items are only activated when requested; focus is only moved with activation.
    pub fn add_item(
        &mut self,
        item: Box<dyn ItemHandle>,
        activate: bool,
        focus_item: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let existing_item_index = self
            .items
            .iter()
            .position(|existing_item| existing_item.item_id() == item.item_id());

        if let Some(existing_item_index) = existing_item_index {
            if activate {
                self.activate_item(existing_item_index, focus_item, window, cx);
            }
        } else {
            self.items.push(item);
            cx.notify();

            if activate {
                self.activate_item(self.items.len() - 1, focus_item, window, cx);
            }

            self.close_items_on_item_open(window, cx);
        }
    }

    pub fn items_len(&self) -> usize {
        self.items.len()
    }

    pub fn items(&self) -> impl DoubleEndedIterator<Item = &Box<dyn ItemHandle>> {
        self.items.iter()
    }

    pub fn active_item(&self, cx: &App) -> Option<Box<dyn ItemHandle>> {
        let item_id = self.stack.read(cx).current()?.entity_id();
        self.items.iter().find(|item| item.item_id() == item_id).cloned()
    }

    /// Removes an item from the cache and the stack's back/forward entries.
    /// Removing the current page reveals the page beneath, or leaves the stack empty.
    /// An empty pane remains open and receives focus if the removed item had it.
    pub fn remove_item(&mut self, item_id: EntityId, focus_item: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item_index) = self.items.iter().position(|item| item.item_id() == item_id) else {
            return;
        };
        let was_active = self
            .stack
            .read(cx)
            .current()
            .is_some_and(|view| view.entity_id() == item_id);
        let should_focus = focus_item
            || self.items[item_index]
                .item_focus_handle(cx)
                .contains_focused(window, cx);
        let mut item = self.items.remove(item_index);
        self.stack.update(cx, |stack, cx| stack.remove(item_id, cx));
        item.on_removed(cx);

        if was_active {
            self.update_title_bar(window, cx);
            self.update_status_bar(window, cx);
        }
        if should_focus {
            self.focus_active_item(window, cx);
        }
        cx.notify();
    }

    /// Pushes a cached item's entity onto the navigation stack.
    pub fn push(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(item) = self.items.get(index) {
            let view = item.to_any_view();
            self.stack.update(cx, |stack, cx| {
                if stack
                    .current()
                    .is_none_or(|current| current.entity_id() != view.entity_id())
                {
                    stack.push(view, NavMotion::Animated, cx);
                }
            });
        }
    }

    pub fn activate_item(&mut self, index: usize, focus_item: bool, window: &mut Window, cx: &mut Context<Self>) {
        if index < self.items.len() {
            self.push(index, cx);

            self.update_title_bar(window, cx);
            self.update_status_bar(window, cx);

            if focus_item {
                self.focus_active_item(window, cx);
            }

            cx.notify();
        }
    }

    fn close_items_on_item_open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(max_items) = self.max_items else {
            return;
        };

        while self.items.len() > max_items.get() {
            let active_id = self.stack.read(cx).current().map(|view| view.entity_id());
            let oldest_inactive = self
                .items
                .iter()
                .find(|item| Some(item.item_id()) != active_id)
                .unwrap();
            self.remove_item(oldest_inactive.item_id(), false, window, cx);
        }
    }

    pub fn focus_active_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active_item) = self.active_item(cx) {
            let focus_handle = active_item.item_focus_handle(cx);
            window.focus(&focus_handle, cx);
        } else {
            self.focus_handle.focus(window, cx);
        }
    }

    pub(crate) fn update_title_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let workspace = self.workspace.clone();
        let active_item = self.active_item(cx);

        window.defer(cx, move |window, cx| {
            let Ok(Some(title_bar)) = workspace.read_with(cx, |workspace, _| workspace.title_bar.clone()) else {
                return;
            };

            title_bar.update(cx, move |title_bar, cx| {
                title_bar.set_active_item(active_item, window, cx);
            });
        });
    }

    pub fn update_status_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let workspace = self.workspace.clone();
        let active_item = self.active_item(cx);

        window.defer(cx, move |window, cx| {
            let Ok(Some(status_bar)) = workspace.read_with(cx, |workspace, _| workspace.status_bar.clone()) else {
                return;
            };

            status_bar.update(cx, move |status_bar, cx| {
                status_bar.set_active_item(active_item, window, cx);
            });
        });
    }
}

impl Focusable for Pane {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Pane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().when_else(
            self.stack.read(cx).is_empty(),
            |pane| {
                pane.child(
                    Empty::new()
                        .size_full()
                        .header(EmptyHeader::new().title(EmptyTitle::new().child("No open items")))
                        .into_any_element(),
                )
            },
            |pane| {
                pane.child(
                    NavStack::new(&self.stack)
                        .w_full()
                        .h_full()
                        .overflow_hidden()
                        .transition(Transition::new(Duration::from_millis(220)))
                        .item(|page, _, _| {
                            let offset = match (page.phase(), page.operation()) {
                                (PresencePhase::Entering, Some(NavOperation::Push | NavOperation::Replace)) => {
                                    1.0 - page.progress()
                                }
                                (PresencePhase::Exiting, Some(NavOperation::Pop)) => page.progress(),
                                (PresencePhase::Exiting, Some(NavOperation::Push)) => -0.3 * page.progress(),
                                (PresencePhase::Entering, Some(NavOperation::Pop)) => -0.3 * (1.0 - page.progress()),
                                _ => 0.0,
                            };
                            page.left(relative(offset)).into_any_element()
                        })
                        .into_any_element(),
                )
            },
        )
    }
}
