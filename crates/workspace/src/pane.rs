use std::time::Duration;

use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement, Render, Styled, Window,
    base::{NavOperation, NavStack, NavStackState, PresencePhase, Transition},
    component::empty::{Empty, EmptyHeader, EmptyTitle},
    div,
    prelude::FluentBuilder,
    relative,
};

use crate::item::ItemHandle;

pub struct Pane {
    items: Vec<Box<dyn ItemHandle>>,
    focus_handle: FocusHandle,
    stack: Entity<NavStackState>,
}

impl Pane {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let stack = cx.new(|_cx| NavStackState::new());
        cx.observe(&stack, |_, _, cx| cx.notify()).detach();

        Self {
            items: Vec::new(),
            stack,
            focus_handle: cx.focus_handle(),
        }
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
