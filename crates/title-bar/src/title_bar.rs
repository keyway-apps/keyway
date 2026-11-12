use gpui_kit::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled, WeakEntity,
    Window, WindowControlArea, base::input::InputState, component::input::Input, div, rgb,
};
use i18n::tr;
use workspace::Workspace;

rust_i18n::i18n!("../../locales", fallback = "en");

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        let item = cx.new(|cx| TitleBar::new(workspace, window, cx));
        workspace.set_titlebar_item(item.into(), cx);
    })
    .detach();
}

pub struct TitleBar {
    input_state: Entity<InputState>,
    workspace: WeakEntity<Workspace>,
}

impl TitleBar {
    pub fn new(workspace: &Workspace, window: &mut Window, cx: &mut Context<TitleBar>) -> Self {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder(tr!("title_bar.search_placeholder")));

        Self {
            input_state,
            workspace: workspace.weak_handle(),
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_16()
            .flex()
            .items_center()
            .gap_2()
            .px_4()
            .border_b_1()
            .border_color(rgb(0xE7E7E7))
            .window_control_area(WindowControlArea::Drag)
            .child(
                div().h_12().flex().min_w_0().flex_1().items_center().occlude().child(
                    Input::new(&self.input_state)
                        .text_lg()
                        .appearance(false)
                        .cleanable(true)
                        .px_0(),
                ),
            )
    }
}
