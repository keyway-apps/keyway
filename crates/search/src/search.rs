use gpui_kit::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, SharedString, Styled,
    WeakEntity, Window,
    base::{h_flex, input::InputState},
    component::input::Input,
};
use i18n::tr;
use workspace::{Workspace, item::ItemHandle, title_bar::TitleItemView};

rust_i18n::i18n!("../../locales", fallback = "en");

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };

        if let Some(title_bar) = workspace.title_bar() {
            let item = cx.new(|cx| SearchBar::new(workspace.weak_handle(), window, cx));
            title_bar.update(cx, |title_bar, cx| {
                title_bar.add_left_item(item, window, cx);
            })
        }
    })
    .detach();
}

pub struct SearchBar {
    pub(crate) input_state: Entity<InputState>,
}

impl SearchBar {
    fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<SearchBar>) -> Self {
        let placeholder: SharedString = tr!("search_bar.placeholder").into();
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder.clone()));

        Self { input_state }
    }
}

impl TitleItemView for SearchBar {
    fn set_active_item(&mut self, active_item: Option<&dyn ItemHandle>, window: &mut Window, cx: &mut Context<Self>) {
        //
    }

    fn hide(&self, cx: &App) -> bool {
        false
    }
}

impl Render for SearchBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().h_12().flex().min_w_0().flex_1().occlude().child(
            Input::new(&self.input_state)
                .text_lg()
                .appearance(false)
                .cleanable(true)
                .px_0(),
        )
    }
}
