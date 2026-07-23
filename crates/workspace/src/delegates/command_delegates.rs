use gpui::{App, Context, IntoElement, SharedString, Window, div, prelude::*};
use gpui_component::{IndexPath, list::{ListDelegate, ListItem, ListState}};

use keyway_core::Command;

use super::{command_filter::CommandFilter, section_manager::SectionManager};

pub struct CommandListDelegate {
    commands: Vec<Command>,
    filter: CommandFilter,
    sections: SectionManager,
    query: String,
    selected: Option<IndexPath>,
}

impl CommandListDelegate {
    pub fn new(commands: Vec<Command>) -> Self {
        let mut delegate = Self {
            commands: sorted_commands(commands),
            filter: CommandFilter::default(),
            sections: SectionManager::default(),
            query: String::new(),
            selected: None,
        };
        delegate.refilter();
        delegate
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.refilter();
        self.selected = None;
    }

    pub fn replace_commands(&mut self, commands: Vec<Command>) {
        self.commands = sorted_commands(commands);
        self.refilter();
        self.selected = None;
    }

    pub fn selected_command(&self) -> Option<&Command> {
        let selected = self.selected?;
        let section = self.sections.section_type_at(selected.section)?;
        let index = self.sections.filtered_index(section, selected.row)?;
        self.commands.get(index)
    }

    fn refilter(&mut self) {
        let filtered = self.filter.filter_with_scores(&self.commands, &self.query);
        self.sections
            .update(filtered, self.commands.len(), &self.query);
    }
}

fn sorted_commands(mut commands: Vec<Command>) -> Vec<Command> {
    commands.sort_by(|left, right| {
        left.title
            .to_lowercase()
            .cmp(&right.title.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
    commands
}

impl ListDelegate for CommandListDelegate {
    type Item = ListItem;

    fn sections_count(&self, _cx: &App) -> usize {
        self.sections.sections_count()
    }

    fn items_count(&self, section: usize, _cx: &App) -> usize {
        self.sections
            .section_type_at(section)
            .map(|section| self.sections.section_item_count(section))
            .unwrap_or(0)
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let section = self.sections.section_type_at(ix.section)?;
        let command_index = self.sections.filtered_index(section, ix.row)?;
        let command = self.commands.get(command_index)?;
        let selected = self.selected == Some(ix);

        let mut content = div()
            .flex()
            .flex_col()
            .child(SharedString::from(command.title.to_owned()));
        if let Some(subtitle) = command.subtitle.as_ref().or(command.description.as_ref()) {
            content = content.child(
                div()
                    .text_sm()
                    .text_color(gpui::rgb(0x6b7280))
                    .child(SharedString::from(subtitle.to_owned())),
            );
        }
        Some(
            ListItem::new(("command-item", (ix.section * 1_000_000 + ix.row) as u64))
                .selected(selected)
                .child(content),
        )
    }

    fn render_section_header(
        &mut self,
        section: usize,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) -> Option<impl IntoElement> {
        let section = self.sections.section_type_at(section)?;
        Some(div().px_3().py_1().text_xs().text_color(gpui::rgb(0x6b7280)).child(section.title()))
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        self.selected = ix;
    }
}
