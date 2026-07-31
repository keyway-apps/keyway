use gpui::{App, Context, IntoElement, Styled, Window};
use gpui_component::{
    IndexPath,
    list::{ListDelegate, ListItem, ListState},
};

use module::{Command, AnyDynamic};

use crate::{dynamic::DynamicItems, filter::CommandFilter, section::SectionManager};

pub struct CommandListDelegate {
    items: Vec<Command>,
    selected_index: Option<usize>,
    query: String,
    dynamic: DynamicItems,
    filter: CommandFilter,
    section: SectionManager,
}

impl CommandListDelegate {
    pub fn new(commands: Vec<Command>, dynamics: Vec<AnyDynamic>) -> Self {
        let len = commands.len();

        let section = SectionManager::new(true);

        Self {
            items: commands,
            selected_index: if len > 0 { Some(0) } else { None },
            query: String::new(),
            dynamic: DynamicItems::new(dynamics),
            filter: Default::default(),
            section,
        }
    }

    pub fn update(&mut self, commands: Vec<Command>) {
        self.items = commands;
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query.clone();
        self.process_query(&query);
    }

    pub fn process_query(&mut self, query: &str) {
        self.filter_items();
    }

    pub fn filter_items(&mut self) {
        let query = &self.query;
        let commands = &self.items;

        let filtered = self.filter.filter_with_scores(commands, query);

        if filtered.len() > 0 {
            // TODO 匹配 dynamic
        }

        self.section.update_with_scores(&self.items, &filtered, false);
    }

    pub fn get_item_at(&self, global_index: usize) {

    }
}

impl ListDelegate for CommandListDelegate {
    type Item = ListItem;

    fn sections_count(&self, _cx: &App) -> usize {
        self.section.sections_count()
    }

    fn items_count(&self, section: usize, _cx: &App) -> usize {
        todo!()
    }

    fn render_item(
        &mut self,
        index: IndexPath,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        todo!()
    }

    fn render_section_header(
        &mut self,
        section: usize,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<impl IntoElement> {
        Some(ListItem::new(("command-item", 0_u32)).py_0().px_0())
    }

    fn set_selected_index(
        &mut self,
        index: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        todo!()
    }
}
