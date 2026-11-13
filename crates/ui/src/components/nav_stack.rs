// Adapted from gpui-base 0.9.1, src/nav_stack.rs (Apache-2.0).
// Upstream: https://github.com/longbridge/gpui-kit
// Revision: 39ab849a93b5f15507b969164d49891db8d5b470.
// License: nav_stack/LICENSE-APACHE.
// Local changes add destructive pop, in-place history removal, and immediate
// release of outgoing views when no animation is requested.

use std::{rc::Rc, time::Duration};

use gpui_kit::{
    AnyElement, AnyView, App, Context, Div, ElementId, Entity, EntityId, EventEmitter, InteractiveElement as _,
    IntoElement, ParentElement as _, RenderOnce, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};

use gpui_kit::base::{History, Presence, PresencePhase, StyledExt as _, Transition};

pub use gpui_kit::base::{NavMotion, NavOperation};

/// Emitted by [`NavStackState`] after the stack changed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavStackEvent {
    Pushed,
    Popped,
    Forwarded,
    Replaced,
    Removed,
    Cleared,
}

/// The view leaving the stack, kept mounted until its exit transition
/// finishes. The element samples that transition as a [`Presence`] keyed by
/// the view, so an interrupted change reverses from where it is.
#[derive(Clone)]
struct Transit {
    outgoing: AnyView,
    /// The position the outgoing view had on the stack.
    index: usize,
    operation: NavOperation,
}

/// A stack of views, one visible at a time, with the pages popped off it
/// kept for `forward`.
///
/// This is SwiftUI's `NavigationStack`, Qt's `StackView` and WinUI's
/// `Frame`: navigation between pages. Underneath it is a [`History`] of
/// views from the root through the current page. A popped page becomes a
/// forward entry until a push discards that forward branch, which is what
/// WinUI's `BackStack` and `ForwardStack` do.
///
/// The stack owns which view is current and the lifecycle of a change: after
/// a push, pop or replace, the outgoing view stays mounted until the
/// [`NavStack`]'s transition finishes, so the application can animate it.
/// The views themselves, and what a transition looks like, belong to the
/// application.
///
/// `pop` keeps the root, as Qt's `StackView` and UIKit's navigation controller
/// do; `clear` is the way to empty the stack. A back button is shown when
/// `depth() > 1`, a forward button when `forward_views()` is not empty.
/// [`pop_without_history`](Self::pop_without_history) discards the popped entry;
/// [`remove`](Self::remove) prunes a view from both branches in place.
pub struct NavStackState {
    history: History<NavEntry>,
    transit: Option<Transit>,
}

/// A view on the stack. A page pushed twice is two separate entries.
#[derive(Clone)]
struct NavEntry {
    view: AnyView,
}

impl NavEntry {
    fn new(view: impl Into<AnyView>) -> Self {
        Self { view: view.into() }
    }
}

impl EventEmitter<NavStackEvent> for NavStackState {}

impl Default for NavStackState {
    fn default() -> Self {
        Self::new()
    }
}

impl NavStackState {
    pub fn new() -> Self {
        Self {
            history: History::new(),
            transit: None,
        }
    }

    /// The number of views on the stack.
    pub fn depth(&self) -> usize {
        self.history.entries().len()
    }

    pub fn is_empty(&self) -> bool {
        self.history.entries().len() == 0
    }

    /// The view on top of the stack, which is the one shown once any
    /// transition has finished.
    pub fn current(&self) -> Option<&AnyView> {
        self.history.current().map(|entry| &entry.view)
    }

    /// Every view on the stack, root first.
    pub fn views(&self) -> impl ExactSizeIterator<Item = &AnyView> {
        self.history.entries().map(|entry| &entry.view)
    }

    /// The views popped since the last push, nearest first: the one
    /// `forward` would bring back is the first.
    pub fn forward_views(&self) -> impl ExactSizeIterator<Item = &AnyView> {
        self.history.forward_entries().map(|entry| &entry.view)
    }

    /// Pushes `view` on top of the stack and discards the forward views.
    ///
    /// Into an empty stack this is immediate, like Qt's `initialItem`. Over
    /// an existing top it starts a [`NavOperation::Push`] transition, unless
    /// `motion` is [`NavMotion::Immediate`].
    pub fn push(&mut self, view: impl Into<AnyView>, motion: NavMotion, cx: &mut Context<Self>) {
        let outgoing = self.top();
        self.history.push(NavEntry::new(view));
        self.finish(outgoing, NavOperation::Push, motion, NavStackEvent::Pushed, cx);
    }

    /// Pops the top view and returns it, starting a [`NavOperation::Pop`]
    /// transition to the view below. The view waits in `forward_views`.
    ///
    /// The root is never popped: this returns `None` at a depth of one or
    /// less.
    pub fn pop(&mut self, motion: NavMotion, cx: &mut Context<Self>) -> Option<AnyView> {
        if self.depth() <= 1 {
            return None;
        }
        let popped = self.top()?;
        self.history.back()?;
        self.finish(
            Some(popped.clone()),
            NavOperation::Pop,
            motion,
            NavStackEvent::Popped,
            cx,
        );
        Some(popped.0)
    }

    /// Pops the current entry without adding it to forward history.
    ///
    /// Keeps the root and any existing forward entries. Earlier visits to the
    /// same view are kept too; use `remove` to delete every visit. An animated
    /// pop retains the outgoing view only until its exit transition finishes.
    pub fn pop_without_history(&mut self, motion: NavMotion, cx: &mut Context<Self>) -> Option<AnyView> {
        if self.depth() <= 1 {
            return None;
        }
        let index = self.depth() - 1;
        let popped = self.history.remove_current()?.view;
        self.finish(
            Some((popped.clone(), index)),
            NavOperation::Pop,
            motion,
            NavStackEvent::Popped,
            cx,
        );
        Some(popped)
    }

    /// Removes every occurrence of a view from back and forward history.
    ///
    /// Prunes in place and collapses adjacent visits to the same view. Removing
    /// the current view reveals the previous surviving view immediately; if no
    /// root survives, the forward branch is cleared. Removing an inactive view
    /// preserves unrelated transitions, while removing an outgoing view also
    /// releases its transition. Returns whether any stack reference changed.
    pub fn remove(&mut self, view_id: EntityId, cx: &mut Context<Self>) -> bool {
        let current_id = self.current().map(|view| view.entity_id());
        let mut removed = false;
        self.history.retain(|entry| {
            if entry.view.entity_id() == view_id {
                removed = true;
                false
            } else {
                true
            }
        });
        let removed_outgoing = self
            .transit
            .as_ref()
            .is_some_and(|transit| transit.outgoing.entity_id() == view_id);
        if !removed && !removed_outgoing {
            return false;
        }

        if self.is_empty() {
            self.history.clear();
        } else if removed {
            self.collapse_adjacent_visits();
        }

        if removed_outgoing || current_id != self.current().map(|view| view.entity_id()) {
            self.transit = None;
        } else if let Some(transit) = &mut self.transit {
            // History pruning can move the mounted views without restarting
            // their Presence animation. Keep their stack positions accurate.
            transit.index = match transit.operation {
                NavOperation::Push => self
                    .history
                    .entries()
                    .rposition(|entry| entry.view.entity_id() == transit.outgoing.entity_id())
                    .unwrap_or(0),
                NavOperation::Pop => self.history.entries().len(),
                NavOperation::Replace => self.history.entries().len().saturating_sub(1),
            };
        }
        cx.emit(NavStackEvent::Removed);
        cx.notify();
        true
    }

    fn collapse_adjacent_visits(&mut self) {
        let current_id = self.current().map(|view| view.entity_id());
        let back_len = self.depth();
        let repeated_current = self
            .history
            .forward_entries()
            .take_while(|entry| Some(entry.view.entity_id()) == current_id)
            .count();
        let forward_end = back_len + self.history.forward_entries().len() - repeated_current;
        let mut index = 0;
        let mut previous_id = None;
        // History::retain visits back entries root-first, then forward entries
        // furthest-first. Collapse each branch and trim the nearest forward
        // visits if pruning made them identical to the current view.
        self.history.retain(|entry| {
            let position = index;
            index += 1;
            if position == back_len {
                previous_id = None;
            }
            let id = entry.view.entity_id();
            let keep = position < forward_end && previous_id != Some(id);
            previous_id = Some(id);
            keep
        });
    }

    /// Pops every view above the root in one [`NavOperation::Pop`]
    /// transition from the previous top, and returns them root-side first.
    pub fn pop_to_root(&mut self, motion: NavMotion, cx: &mut Context<Self>) -> Vec<AnyView> {
        let outgoing = self.top();
        let mut popped = Vec::new();
        while self.depth() > 1 {
            let Some(view) = self.current().cloned() else {
                break;
            };
            if self.history.back().is_none() {
                break;
            }
            popped.push(view);
        }
        if popped.is_empty() {
            return popped;
        }
        self.finish(outgoing, NavOperation::Pop, motion, NavStackEvent::Popped, cx);
        popped.reverse();
        popped
    }

    /// Brings back the most recently popped view, starting a
    /// [`NavOperation::Push`] transition over the current top, and returns
    /// it. `None` when nothing has been popped since the last push.
    pub fn forward(&mut self, motion: NavMotion, cx: &mut Context<Self>) -> Option<AnyView> {
        let outgoing = self.top();
        let view = self.history.forward()?.view;
        self.finish(outgoing, NavOperation::Push, motion, NavStackEvent::Forwarded, cx);
        Some(view)
    }

    /// Swaps the top view for `view` and returns the one replaced, starting a
    /// [`NavOperation::Replace`] transition. The forward views are kept. On
    /// an empty stack this is a push.
    pub fn replace(&mut self, view: impl Into<AnyView>, motion: NavMotion, cx: &mut Context<Self>) -> Option<AnyView> {
        let Some(replaced) = self.top() else {
            self.push(view, motion, cx);
            return None;
        };
        self.history.replace_current(NavEntry::new(view));
        self.finish(
            Some(replaced.clone()),
            NavOperation::Replace,
            motion,
            NavStackEvent::Replaced,
            cx,
        );
        Some(replaced.0)
    }

    /// Empties the stack and the forward views immediately, abandoning any
    /// running transition.
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.history.clear();
        self.transit = None;
        cx.emit(NavStackEvent::Cleared);
        cx.notify();
    }

    /// The top view with its position, before an operation moves it.
    fn top(&self) -> Option<(AnyView, usize)> {
        let index = self.depth().checked_sub(1)?;
        self.current().cloned().map(|view| (view, index))
    }

    /// Records the change: `outgoing` stays mounted until its exit
    /// transition finishes, or not at all for an immediate change. A change
    /// already in transit is superseded, so at most one outgoing view is ever
    /// mounted; the element reverses its presence from wherever it is.
    fn finish(
        &mut self,
        outgoing: Option<(AnyView, usize)>,
        operation: NavOperation,
        motion: NavMotion,
        event: NavStackEvent,
        cx: &mut Context<Self>,
    ) {
        self.transit = outgoing
            .filter(|(view, _)| {
                motion == NavMotion::Animated
                    && self
                        .current()
                        .is_some_and(|current| current.entity_id() != view.entity_id())
            })
            .map(|(outgoing, index)| Transit {
                outgoing,
                index,
                operation,
            });
        cx.emit(event);
        cx.notify();
    }
}

type ItemRenderer = Rc<dyn Fn(NavPage, &mut Window, &mut App) -> AnyElement>;

/// An unstyled host for a [`NavStackState`].
///
/// The container is positioned so that the two views of a transition can
/// overlap; each mounted view is handed to the `item` renderer as a
/// [`NavPage`] that already fills the container. Everything else — size,
/// clipping, background, and how a transition moves — is the application's.
///
/// Without a `transition` the stack switches views immediately, as it also
/// does under reduced motion.
#[derive(IntoElement)]
pub struct NavStack {
    base: Div,
    style: StyleRefinement,
    state: Entity<NavStackState>,
    transition: Option<Transition>,
    render_item: Option<ItemRenderer>,
}

impl NavStack {
    pub fn new(state: &Entity<NavStackState>) -> Self {
        Self {
            base: div(),
            style: StyleRefinement::default(),
            state: state.clone(),
            transition: None,
            render_item: None,
        }
    }

    /// The timing every push, pop and replace runs under.
    pub fn transition(mut self, transition: Transition) -> Self {
        self.transition = Some(transition);
        self
    }

    /// Renders each mounted view. The item is already positioned to fill the
    /// container; refine it to move or fade the view by its phase and
    /// progress, then return it.
    pub fn item(mut self, render: impl Fn(NavPage, &mut Window, &mut App) -> AnyElement + 'static) -> Self {
        self.render_item = Some(Rc::new(render));
        self
    }
}

impl Styled for NavStack {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for NavStack {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (current, depth, transit) = {
            let state = self.state.read(cx);
            (state.current().cloned(), state.depth(), state.transit.clone())
        };
        let immediate = cx.reduce_motion() || self.transition.is_none();
        let transition = if immediate {
            Transition::new(Duration::ZERO)
        } else {
            self.transition
                .clone()
                .unwrap_or_else(|| Transition::new(Duration::ZERO))
        };

        // The outgoing view's presence is the change's clock: its exit runs
        // 1 → 0, reversing if the change is interrupted, and both pages of
        // the change read their progress from it.
        let change = transit.and_then(|transit| {
            let sample = Presence::new(page_id(&transit.outgoing), false)
                .transition(transition.clone())
                .sample(window, cx);
            if sample.should_render() {
                Some((transit, 1.0 - sample.progress))
            } else {
                self.state.update(cx, |state, _| state.transit = None);
                None
            }
        });

        // The current view's presence is sampled too, so a view brought back
        // by `forward` or revealed by `pop` starts its next exit from present.
        // With nothing changing it settles on the spot; the root does not
        // animate in.
        if let Some(current) = &current {
            let transition = if change.is_some() {
                transition
            } else {
                Transition::new(Duration::ZERO)
            };
            Presence::new(page_id(current), true)
                .transition(transition)
                .sample(window, cx);
        }

        let mut items = Vec::with_capacity(3);
        if let Some(current) = current {
            let index = depth - 1;
            match change {
                Some((transit, progress)) => {
                    let current = NavPage::new(
                        current,
                        index,
                        PresencePhase::Entering,
                        Some(transit.operation),
                        progress,
                    );
                    let outgoing = NavPage::new(
                        transit.outgoing,
                        transit.index,
                        PresencePhase::Exiting,
                        Some(transit.operation),
                        progress,
                    );
                    // A pushed or replacing view paints over what it covers; a
                    // popped view paints over what it reveals.
                    match transit.operation {
                        NavOperation::Push | NavOperation::Replace => {
                            items.push(outgoing);
                            items.push(current);
                        }
                        NavOperation::Pop => {
                            items.push(current);
                            items.push(outgoing);
                        }
                    }
                }
                None => items.push(NavPage::new(current, index, PresencePhase::Present, None, 1.0)),
            }
        }
        let changing = items.len() > 1;

        let render_item = self.render_item;
        self.base
            .relative()
            .refine_style(&self.style)
            .children(items.into_iter().map(|item| match &render_item {
                Some(render) => render(item, window, cx),
                None => item.into_any_element(),
            }))
            // Neither page takes pointer input while the change runs: the
            // outgoing one is on its way out, and the incoming one is not yet
            // where it will be.
            .when(changing, |this| this.child(div().absolute().inset_0().occlude()))
    }
}

fn page_id(view: &AnyView) -> ElementId {
    ("nav-stack", view.entity_id()).into()
}

/// One mounted view of a [`NavStack`], handed to the item renderer.
///
/// The item fills its container. Its readers describe where the view is in
/// the change that is running, so the renderer can move it: `phase` says
/// whether it is arriving, settled, or leaving; `operation` says which
/// change; `progress` runs from `0.0` to `1.0` over the transition, already
/// eased, and is shared by both views of one change.
#[derive(IntoElement)]
pub struct NavPage {
    base: Div,
    style: StyleRefinement,
    view: AnyView,
    index: usize,
    phase: PresencePhase,
    operation: Option<NavOperation>,
    progress: f32,
}

impl NavPage {
    fn new(view: AnyView, index: usize, phase: PresencePhase, operation: Option<NavOperation>, progress: f32) -> Self {
        Self {
            base: div(),
            style: StyleRefinement::default(),
            view,
            index,
            phase,
            operation,
            progress,
        }
    }

    pub fn view(&self) -> &AnyView {
        &self.view
    }

    /// The view's position on the stack, root first. A view on its way out
    /// keeps the position it had.
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn phase(&self) -> PresencePhase {
        self.phase
    }

    /// The change in progress, or `None` once the stack has settled.
    pub fn operation(&self) -> Option<NavOperation> {
        self.operation
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }
}

impl Styled for NavPage {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for NavPage {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.base
            .absolute()
            .inset_0()
            .child(self.view)
            .refine_style(&self.style)
    }
}
