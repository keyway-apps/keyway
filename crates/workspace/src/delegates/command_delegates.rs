use gpui::{App, Context, IntoElement, SharedString, Window};
use gpui_component::{
    IndexPath,
    list::{ListDelegate, ListItem, ListState},
};
use keyway_core::Command;
use keyway_ui::{ItemList, ItemListSection, ItemListState};

use super::{command_filter::CommandFilter, section_manager::SectionManager};

struct CommandListItem(Command);

impl ItemList for CommandListItem {
    fn title(&self) -> SharedString {
        self.0.title.clone().into()
    }

    fn subtitle(&self) -> Option<SharedString> {
        self.0
            .subtitle
            .as_ref()
            .or(self.0.description.as_ref())
            .cloned()
            .map(Into::into)
    }

    fn icon(&self) -> Option<SharedString> {
        self.0.icon.clone().map(Into::into)
    }
}

pub struct CommandListDelegate {
    list: ItemListState<CommandListItem>,
    filter: CommandFilter,
    sections: SectionManager,
    query: String,
}

impl CommandListDelegate {
    pub fn new(commands: Vec<Command>) -> Self {
        let mut delegate = Self {
            list: ItemListState::new(command_items(commands)),
            filter: CommandFilter::default(),
            sections: SectionManager::default(),
            query: String::new(),
        };
        delegate.refilter();
        delegate
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.refilter();
    }

    pub fn replace_commands(&mut self, commands: Vec<Command>) {
        self.list.replace_items(command_items(commands));
        self.refilter();
    }

    pub fn selected_command(&self) -> Option<&Command> {
        self.list.selected_item().map(|item| &item.0)
    }

    fn refilter(&mut self) {
        let filtered = self
            .filter
            .filter_with_scores(self.list.items().iter().map(|item| &item.0), &self.query);
        self.sections
            .update(filtered, self.list.items().len(), &self.query);

        let sections = self.sections.sections().into_iter().map(|section| {
            let indices = (0..self.sections.section_item_count(section))
                .filter_map(|row| self.sections.filtered_index(section, row));
            ItemListSection::new(section.title(), indices)
        });
        self.list.set_sections(sections);
    }
}

fn command_items(mut commands: Vec<Command>) -> Vec<CommandListItem> {
    commands.sort_by(|left, right| {
        left.title
            .to_lowercase()
            .cmp(&right.title.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
    commands.into_iter().map(CommandListItem).collect()
}

impl ListDelegate for CommandListDelegate {
    type Item = ListItem;

    fn sections_count(&self, _cx: &App) -> usize {
        self.list.sections_count()
    }

    fn items_count(&self, section: usize, _cx: &App) -> usize {
        self.list.items_count(section)
    }

    fn render_item(
        &mut self,
        index: IndexPath,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        self.list.render_item(index, cx)
    }

    fn render_section_header(
        &mut self,
        section: usize,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<impl IntoElement> {
        self.list.render_section_header(section, cx)
    }

    fn set_selected_index(
        &mut self,
        index: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        self.list.set_selected_index(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtering_rebuilds_shared_sections_and_selection_mapping() {
        let mut delegate = CommandListDelegate::new(vec![
            Command::new("zulu", "Zulu"),
            Command::new("alpha", "Alpha"),
        ]);

        assert_eq!(
            delegate
                .list
                .item_at(IndexPath::new(0).section(0))
                .map(|item| item.0.title.as_str()),
            Some("Alpha")
        );

        delegate.set_query("zulu");
        let best_match = IndexPath::new(0).section(0);
        delegate.list.set_selected_index(Some(best_match));

        assert_eq!(
            delegate
                .selected_command()
                .map(|command| command.id.as_str()),
            Some("zulu")
        );
    }
}
