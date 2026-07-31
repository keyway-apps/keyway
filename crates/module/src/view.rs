use gpui::{Context, IntoElement, Window};

pub trait ViewRender: 'static + Sized {
    fn render(
        &mut self,
        context: &mut ViewContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement;
}

pub struct ViewContext {}
