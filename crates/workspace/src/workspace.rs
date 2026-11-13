use std::sync::Arc;

use collections::pinnedset::PinnedSet;
use gpui_kit::component::Root;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    AnyWindowHandle, App, AppContext, Context, Entity, Global, IntoElement, ParentElement, Render, Styled,
    Subscription, Task, WeakEntity, Window, WindowHandle, WindowOptions, div,
};
use ui::NavMotion;

use crate::item::ItemHandle;
use crate::pane::Pane;
use crate::status_bar::StatusBar;
use crate::title_bar::TitleBar;

pub mod actions;
pub mod item;
pub mod pane;
pub mod searchable;
pub mod status_bar;
pub mod title_bar;

pub struct AppState {
    pub workspace_store: Entity<WorkspaceStore>,
}

impl AppState {
    pub fn global(cx: &App) -> Arc<Self> {
        cx.global::<GlobalAppState>().0.clone()
    }

    pub fn try_global(cx: &App) -> Option<Arc<Self>> {
        cx.try_global::<GlobalAppState>().map(|state| state.0.clone())
    }

    pub fn set_global(state: Arc<AppState>, cx: &mut App) {
        cx.set_global(GlobalAppState(state));
    }
}

struct GlobalAppState(Arc<AppState>);

impl Global for GlobalAppState {}

pub struct WorkspaceStore {
    workspaces: PinnedSet<(AnyWindowHandle, WeakEntity<Workspace>)>,
}

impl WorkspaceStore {
    pub fn new() -> Self {
        Self {
            workspaces: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorkspaceId(i64);

pub struct Workspace {
    weak_self: WeakEntity<Self>,
    title_bar: Option<Entity<TitleBar>>,
    status_bar: Option<Entity<StatusBar>>,
    center: Entity<Pane>,
    app_state: Arc<AppState>,
    _subscriptions: Vec<Subscription>,
}

impl Workspace {
    pub fn new(app_state: Arc<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let weak_handle = cx.entity().downgrade();

        let center = cx.new(|cx| Pane::new(weak_handle.clone(), cx));

        let title_bar = cx.new(|cx| TitleBar::new(weak_handle.clone(), window, cx));
        let status_bar = cx.new(|cx| StatusBar::new(weak_handle.clone(), window, cx));

        let window_handle = window.window_handle();
        app_state.workspace_store.update(cx, |store, _| {
            store.workspaces.insert((window_handle, weak_handle.clone()));
        });

        let subscriptions = vec![
            cx.observe_window_activation(window, |workspace, window, cx| {
                if workspace.is_pinned(window, cx) && !window.is_window_active() {
                    #[cfg(windows)]
                    util::set_native_window_visibility(window, false);
                    #[cfg(not(windows))]
                    window.minimize_window();
                }
            }),
            cx.on_release({
                let weak_handle = weak_handle.clone();
                move |this, cx| {
                    this.app_state.workspace_store.update(cx, move |store, _| {
                        store.workspaces.retain(|(_, weak)| weak != &weak_handle);
                    });
                }
            }),
        ];

        Self {
            weak_self: weak_handle.clone(),
            title_bar: Some(title_bar),
            status_bar: Some(status_bar),
            center,
            app_state,
            _subscriptions: subscriptions,
        }
    }

    pub fn new_local(
        app_state: Arc<AppState>,
        options: WindowOptions,
        cx: &mut App,
    ) -> Task<anyhow::Result<WindowHandle<Root>>> {
        cx.spawn(async move |cx| {
            let window = cx.open_window(options, {
                let app_state = app_state.clone();
                move |window, cx| {
                    let workspace = cx.new(|cx| Workspace::new(app_state, window, cx));
                    cx.new(|cx| Root::new(workspace, window, cx))
                }
            })?;

            Ok(window)
        })
    }

    pub fn title_bar(&self) -> Option<Entity<TitleBar>> {
        self.title_bar.clone()
    }

    pub fn status_bar(&self) -> Option<Entity<StatusBar>> {
        self.status_bar.clone()
    }

    pub fn weak_handle(&self) -> WeakEntity<Self> {
        self.weak_self.clone()
    }

    pub fn for_window(window: &Window, cx: &App) -> Option<Entity<Workspace>> {
        window
            .root::<Root>()
            .flatten()
            .and_then(|root| root.read(cx).view().clone().downcast::<Workspace>().ok())
    }

    pub fn for_root(root: &Root, _cx: &App) -> Option<Entity<Workspace>> {
        root.view().clone().downcast::<Workspace>().ok()
    }

    pub fn is_pinned(&self, window: &Window, cx: &mut App) -> bool {
        self.app_state
            .workspace_store
            .read(cx)
            .workspaces
            .is_pinned(&(window.window_handle(), self.weak_handle()))
    }

    pub fn update_pinned_workspace(&self, window: &Window, cx: &mut App) {
        self.app_state.workspace_store.update(cx, |store, _cx| {
            let window_handle = window.window_handle();
            store.workspaces.set_pinned(Some((window_handle, self.weak_handle())));
        });
    }

    pub fn active_pane(&self) -> &Entity<Pane> {
        &self.center
    }

    pub fn add_item(
        &mut self,
        item: Box<dyn ItemHandle>,
        activate: bool,
        focus_item: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.center
            .update(cx, |pane, cx| pane.add_item(item, activate, focus_item, window, cx))
    }

    /// Returns to the previous Item without adding a forward-history entry.
    /// Keeps Items cached for reuse and retains the root page.
    pub fn go_back(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.center.update(cx, |pane, cx| {
            let popped = pane
                .stack()
                .update(cx, |stack, cx| stack.pop_without_history(NavMotion::Animated, cx));
            if popped.is_none() {
                return false;
            }
            pane.update_title_bar(window, cx);
            pane.update_status_bar(window, cx);
            pane.focus_active_item(window, cx);
            true
        })
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .gap_0()
            .overflow_hidden()
            .when_some(self.title_bar.clone(), |view, bar| view.child(bar))
            .child(div().flex_1().min_h_0().child(self.center.clone()))
            .when_some(self.status_bar.clone(), |view, bar| view.child(bar))
    }
}

pub fn open_new(
    app_state: Arc<AppState>,
    options: WindowOptions,
    cx: &mut App,
    init: impl FnOnce(&mut Workspace, &mut Window, &mut Context<Workspace>) + 'static + Send,
) -> Task<anyhow::Result<()>> {
    let task = Workspace::new_local(app_state, options, cx);
    cx.spawn(async move |cx| {
        let root = task.await?;
        root.update(cx, |root, window, cx| {
            // The Root is already borrowed by WindowHandle::update; use it directly.
            Workspace::for_root(root, cx).map(|workspace| {
                workspace.update(cx, |workspace, cx| init(workspace, window, cx));
            });
        })?;
        Ok(())
    })
}

pub fn active_pinned_window(cx: &mut App) {
    let app_state = AppState::global(cx);
    let Some(window) = app_state
        .workspace_store
        .read(cx)
        .workspaces
        .pinned()
        .map(|(window, _)| *window)
    else {
        return;
    };

    window
        .update(cx, |_, window, _| {
            #[cfg(windows)]
            util::set_native_window_visibility(window, true);
            #[cfg(not(windows))]
            window.minimize_window();
            window.activate_window();
        })
        .ok();
}
