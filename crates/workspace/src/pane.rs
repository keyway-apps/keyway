use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use gpui_kit::{
    App, AppContext, Context, Entity, EntityId, FocusHandle, Focusable, IntoElement, ParentElement, Render, Styled,
    WeakEntity, Window,
    base::{NavMotion, NavOperation, NavStack, NavStackState, PresencePhase, Transition},
    component::empty::{Empty, EmptyHeader, EmptyTitle},
    div,
    prelude::FluentBuilder,
    relative,
};

use crate::{Workspace, item::ItemHandle};

/// An item activation recorded in a pane's navigation history.
pub struct ActivationHistoryEntry {
    pub entity_id: EntityId,
    /// A monotonically increasing activation sequence number within the pane.
    pub timestamp: usize,
}

pub struct Pane {
    // Oldest opened items come first; activation does not change this order.
    items: Vec<Box<dyn ItemHandle>>,
    focus_handle: FocusHandle,
    active_item_index: usize,
    // Activation order is independent of the cache's opening order.
    activation_history: Vec<ActivationHistoryEntry>,
    next_activation_timestamp: Arc<AtomicUsize>,
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
            active_item_index: 0,
            activation_history: Vec::new(),
            next_activation_timestamp: Arc::new(AtomicUsize::new(0)),
            workspace,
            max_items: NonZeroUsize::new(10),
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn active_item_index(&self) -> usize {
        self.active_item_index
    }

    /// Reuses existing items and evicts the oldest inactive items when the cache is full.
    /// Items are only activated when requested; focus is only moved with activation.
    pub fn add_item(
        &mut self,
        item: Box<dyn ItemHandle>,
        focus_item: bool,
        activate: bool,
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
                self.active_item_index += 1;
                self.activate_item(self.active_item_index, focus_item, window, cx);
            }

            self.close_items_on_item_open();
        }
    }

    pub fn items_len(&self) -> usize {
        self.items.len()
    }

    pub fn items(&self) -> impl DoubleEndedIterator<Item = &Box<dyn ItemHandle>> {
        self.items.iter()
    }

    pub fn active_item(&self) -> Option<Box<dyn ItemHandle>> {
        self.items.get(self.active_item_index).cloned()
    }

    /// Deactivates the current item and focuses the previously activated item.
    /// Keeps both items cached and does nothing at the first activation.
    pub fn close_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(previous_activation_index) = self.activation_history.len().checked_sub(2) else {
            return;
        };
        let previous_item_id = self.activation_history[previous_activation_index].entity_id;
        let Some(previous_item_index) = self.items.iter().position(|item| item.item_id() == previous_item_id) else {
            return;
        };

        self.activation_history.pop();
        self.activate_item(previous_item_index, true, window, cx);
    }

    pub fn activate_item(&mut self, index: usize, focus_item: bool, window: &mut Window, cx: &mut Context<Self>) {
        if index < self.items.len() {
            self.active_item_index = index;
            let item_id = self.items[index].item_id();
            if self
                .activation_history
                .last()
                .is_none_or(|entry| entry.entity_id != item_id)
            {
                self.activation_history.push(ActivationHistoryEntry {
                    entity_id: item_id,
                    timestamp: self.next_activation_timestamp.fetch_add(1, Ordering::Relaxed),
                });
            }
            let view = self.items[index].to_any_view();
            self.stack.update(cx, |stack, cx| {
                if stack
                    .current()
                    .is_none_or(|current| current.entity_id() != view.entity_id())
                {
                    stack.replace(view, NavMotion::Animated, cx);
                }
            });
            self.update_title_bar(window, cx);
            self.update_status_bar(window, cx);

            if focus_item {
                self.focus_active_item(window, cx);
            }

            cx.notify();
        }
    }

    fn close_items_on_item_open(&mut self) {
        let Some(max_items) = self.max_items else {
            return;
        };

        while self.items.len() > max_items.get() {
            // Items are ordered oldest first; skip the first item only when it is active.
            let oldest_inactive_index = if self.active_item_index == 0 { 1 } else { 0 };
            let removed_item = self.items.remove(oldest_inactive_index);

            if oldest_inactive_index < self.active_item_index {
                self.active_item_index -= 1;
            }

            self.remove_item_from_activation_history(removed_item.item_id());
        }
    }

    fn remove_item_from_activation_history(&mut self, item_id: EntityId) {
        self.activation_history.retain(|entry| entry.entity_id != item_id);

        // Removing B from A -> B -> A leaves adjacent A entries; keep the newer activation.
        self.activation_history.dedup_by(|later, earlier| {
            if later.entity_id != earlier.entity_id {
                return false;
            }

            earlier.timestamp = later.timestamp;
            true
        });
    }

    pub fn focus_active_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active_item) = self.active_item() {
            let focus_handle = active_item.item_focus_handle(cx);
            window.focus(&focus_handle, cx);
        }
    }

    fn update_title_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let workspace = self.workspace.clone();
        let active_item = self.items.get(self.active_item_index).map(|item| item.boxed_clone());

        window.defer(cx, move |window, cx| {
            let Ok(title_bar) = workspace.read_with(cx, |workspace, _| workspace.title_bar.clone()) else {
                return;
            };

            title_bar.update(cx, move |title_bar, cx| {
                title_bar.set_active_item(active_item, window, cx);
            });
        });
    }

    pub fn update_status_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let workspace = self.workspace.clone();
        let active_item = self.items.get(self.active_item_index).map(|item| item.boxed_clone());

        window.defer(cx, move |window, cx| {
            let Ok(status_bar) = workspace.read_with(cx, |workspace, _| workspace.status_bar.clone()) else {
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
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().when_else(
            self.items.is_empty(),
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
