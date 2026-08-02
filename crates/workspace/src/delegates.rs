use gpui::{App, Context, IntoElement, Pixels, Window, px};
use gpui_component::{
    IndexPath,
    list::{ListDelegate, ListItem, ListState},
};
use module::{AnyDynamic, Command};

use crate::{
    dynamic::DynamicItems,
    filter::CommandFilter,
    render::{
        COMMAND_ITEM_HEIGHT, DYNAMIC_ITEM_HEIGHT, render_command_item, render_dynamic_item,
        render_section_header,
    },
    section::{SectionItem, SectionManager},
};

pub struct CommandListDelegate {
    commands: Vec<Command>,
    selected_index: Option<IndexPath>,
    query: String,
    dynamics: DynamicItems,
    filter: CommandFilter,
    sections: SectionManager,
}

impl CommandListDelegate {
    pub fn new(mut commands: Vec<Command>, dynamics: Vec<AnyDynamic>) -> Self {
        sort_commands(&mut commands);
        let filter = CommandFilter::new(&commands);
        let mut this = Self {
            commands,
            selected_index: None,
            query: String::new(),
            dynamics: DynamicItems::new(dynamics),
            filter,
            sections: SectionManager::default(),
        };
        this.refresh_without_dynamic();
        this
    }

    pub fn update(&mut self, mut commands: Vec<Command>, dynamics: Vec<AnyDynamic>, cx: &mut App) {
        sort_commands(&mut commands);
        self.commands = commands;
        self.dynamics.replace(dynamics);
        self.filter.replace_commands(&self.commands);
        self.process_query(cx);
    }

    pub fn set_query(&mut self, query: String, cx: &mut App) {
        self.query = query;
        self.process_query(cx);
    }

    pub fn selected_index(&self) -> Option<IndexPath> {
        self.selected_index
    }

    fn process_query(&mut self, cx: &mut App) {
        let query = self.query.trim();
        let keyword_matches = self.filter.filter_with_scores(self.commands.iter(), query);

        let (has_dynamic, regex_matches) = if !query.is_empty() && keyword_matches.is_empty() {
            (
                self.dynamics.process_query(query, cx),
                self.filter.regex_matches(query),
            )
        } else {
            self.dynamics.clear();
            (false, Vec::new())
        };

        self.sections.update(
            query,
            keyword_matches,
            regex_matches,
            has_dynamic,
            self.commands.len(),
        );
        self.reset_selection();
    }

    fn refresh_without_dynamic(&mut self) {
        let keyword_matches = self
            .filter
            .filter_with_scores(self.commands.iter(), &self.query);
        self.sections.update(
            &self.query,
            keyword_matches,
            Vec::new(),
            false,
            self.commands.len(),
        );
        self.reset_selection();
    }

    fn reset_selection(&mut self) {
        self.selected_index = self
            .sections
            .first_item()
            .map(|(section, row)| IndexPath::new(row).section(section));
    }
}

fn sort_commands(commands: &mut [Command]) {
    commands.sort_by(|left, right| {
        left.title
            .to_lowercase()
            .cmp(&right.title.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn row_id(index: IndexPath) -> u64 {
    ((index.section as u64) << 32) | index.row as u64
}

impl ListDelegate for CommandListDelegate {
    type Item = ListItem;

    fn sections_count(&self, _cx: &App) -> usize {
        self.sections.sections_count()
    }

    fn items_count(&self, section: usize, _cx: &App) -> usize {
        self.sections.items_count(section)
    }

    fn item_height(&self, index: IndexPath, _cx: &App) -> Option<Pixels> {
        self.sections
            .item_at(index.section, index.row)
            .map(|item| match item {
                SectionItem::Dynamic => px(DYNAMIC_ITEM_HEIGHT),
                SectionItem::Command { .. } => px(COMMAND_ITEM_HEIGHT),
            })
    }

    fn render_item(
        &mut self,
        index: IndexPath,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let selected = self.selected_index == Some(index);
        let id = row_id(index);

        match self.sections.item_at(index.section, index.row)? {
            SectionItem::Dynamic => self
                .dynamics
                .selected()
                .map(|dynamic| render_dynamic_item(id, dynamic, selected, window, cx)),
            SectionItem::Command {
                index: command_index,
                regex_match,
            } => {
                let command = self.commands.get(command_index)?;
                let regex_query = regex_match.then_some(self.query.trim());
                Some(render_command_item(id, command, regex_query, selected, cx))
            }
        }
    }

    fn render_section_header(
        &mut self,
        section: usize,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<impl IntoElement> {
        let section_type = self.sections.section_type_at(section)?;
        let title = match section_type.title() {
            Some(title) => title.to_owned(),
            None => self.dynamics.selected()?.title().to_owned(),
        };
        Some(render_section_header(title, cx))
    }

    fn set_selected_index(
        &mut self,
        index: Option<IndexPath>,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index =
            index.filter(|index| self.sections.item_at(index.section, index.row).is_some());
        cx.notify();
    }
}
