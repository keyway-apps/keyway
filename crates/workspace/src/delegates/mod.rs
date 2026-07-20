use gpui::{App, Context, Window};
use gpui_component::{
    IndexPath,
    list::{ListDelegate, ListItem, ListState},
};

pub struct CommandListDelegate {}

impl CommandListDelegate {
    pub fn new() -> Self {
        Self {}
    }
}

impl ListDelegate for CommandListDelegate {
    type Item = ListItem;

    fn items_count(&self, section: usize, cx: &App) -> usize {
        todo!()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        todo!()
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        todo!()
    }
}
