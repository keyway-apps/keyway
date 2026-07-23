use gpui::{App, Context, Entity};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use keyway_collections::{HashMap, hash_map::Entry};

use crate::GlobalCommandRegistry;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub match_regexes: Option<Vec<String>>,
    pub shortcut: Option<String>,
    pub disabled_by_default: bool,
    pub visible_by_default: bool,
}

impl Command {
    pub fn new<T: Into<String>>(id: T, title: T) -> Command {
        CommandBuilder::new(id, title).build()
    }

    pub fn search_terms(&self) -> Vec<&str> {
        let mut terms = vec![self.title.as_str()];

        self.subtitle
            .as_ref()
            .map(|subtitle| terms.push(subtitle.as_str()));
        self.category
            .as_ref()
            .map(|category| terms.push(category.as_str()));
        self.description
            .as_ref()
            .map(|description| terms.push(description.as_str()));
        self.keywords
            .as_ref()
            .map(|keywords| terms.extend(keywords.iter().map(|keyword| keyword.as_str())));

        terms
    }

    pub fn builder<T: Into<String>>(id: T, title: T) -> CommandBuilder {
        CommandBuilder::new(id, title)
    }
}

impl From<CommandBuilder> for Command {
    fn from(builder: CommandBuilder) -> Self {
        builder.build()
    }
}

pub struct CommandBuilder {
    id: String,
    title: String,
    subtitle: Option<String>,
    category: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    keywords: Option<Vec<String>>,
    match_regexes: Option<Vec<String>>,
    shortcut: Option<String>,
    disabled_by_default: bool,
    visible_by_default: bool,
}

impl CommandBuilder {
    pub fn new<T: Into<String>>(id: T, title: T) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            category: None,
            description: None,
            icon: None,
            keywords: None,
            match_regexes: None,
            shortcut: None,
            disabled_by_default: false,
            visible_by_default: true,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keywords
            .get_or_insert_with(Vec::new)
            .push(keyword.into());
        self
    }

    pub fn keywords<I, K>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = K>,
        K: Into<String>,
    {
        self.keywords
            .get_or_insert_with(Vec::new)
            .extend(keywords.into_iter().map(Into::into));
        self
    }

    pub fn match_regex(mut self, regex: impl Into<String>) -> Self {
        self.match_regexes
            .get_or_insert_with(Vec::new)
            .push(regex.into());
        self
    }

    pub fn match_regexes<I, R>(mut self, regexes: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Into<String>,
    {
        self.match_regexes
            .get_or_insert_with(Vec::new)
            .extend(regexes.into_iter().map(Into::into));
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn disabled_by_default(mut self) -> Self {
        self.disabled_by_default = true;
        self
    }

    pub fn visible_by_default(mut self) -> Self {
        self.visible_by_default = false;
        self
    }

    pub fn build(self) -> Command {
        Command {
            id: self.id,
            title: self.title,
            subtitle: self.subtitle,
            category: self.category,
            description: self.description,
            icon: self.icon,
            keywords: self.keywords,
            match_regexes: self.match_regexes,
            shortcut: self.shortcut,
            disabled_by_default: self.disabled_by_default,
            visible_by_default: self.visible_by_default,
        }
    }
}

pub trait CommandAction:
    Fn(&mut Actions, &mut Context<App>) -> anyhow::Result<(), anyhow::Error> + 'static
{
}
impl<F> CommandAction for F where
    F: Fn(&mut Actions, &mut Context<App>) -> anyhow::Result<(), anyhow::Error> + 'static
{
}
// pub type CommandAction = Fn(&mut Actions, &mut Context<App>) -> anyhow::Result<(), anyhow::Error>;

pub struct Actions {
    query: Option<String>,
}

pub struct CommandRegistry {
    commands: HashMap<String, Command>,
    actions: HashMap<String, Rc<dyn CommandAction>>,
}

impl CommandRegistry {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalCommandRegistry>().0.clone()
    }

    pub(crate) fn new() -> Self {
        Self {
            commands: HashMap::default(),
            actions: HashMap::default(),
        }
    }

    pub fn register_command(
        &mut self,
        cx: &mut Context<Self>,
        command: Command,
        action: impl CommandAction,
    ) {
        let command_id = command.id.clone();
        let inserted = self.insert_command(command);

        if inserted {
            self.actions.insert(command_id, Rc::new(action));
            cx.notify();
        }
    }

    fn insert_command(&mut self, command: Command) -> bool {
        match self.commands.entry(command.id.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(command);
                true
            }
            Entry::Occupied(_) => {
                tracing::warn!(
                    command_id = %command.id,
                    "duplicate command registration detected, skipping"
                );
                false
            }
        }
    }

    pub fn unregister_command(&mut self, command_id: &str, cx: &mut Context<Self>) {
        self.remove_command(command_id).then(|| cx.notify());
    }

    pub fn unregister_commands<I, S>(&mut self, command_ids: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        command_ids
            .into_iter()
            .fold(false, |acc, command_id| {
                self.remove_command(command_id.as_ref()) || acc
            })
            .then(|| cx.notify());
    }

    fn remove_command(&mut self, command_id: &str) -> bool {
        match self.commands.entry(command_id.to_string()) {
            Entry::Vacant(_) => {
                tracing::warn!(
                    command_id,
                    "command unregistration failed, command not found"
                );
                false
            }
            Entry::Occupied(entry) => {
                entry.remove();
                self.actions.remove(command_id);
                true
            }
        }
    }

    pub fn execute_command(&self, command_id: &str, cx: Context<App>) {
        let action = self.actions.get(command_id);
        if let Some(action) = action {
            // action();
        }
    }

    pub fn visible_commands(&self) -> impl Iterator<Item = &Command> {
        self.commands
            .values()
            .filter(|cmd| !cmd.disabled_by_default && cmd.visible_by_default)
    }

    pub fn enabled_commands(&self) -> impl Iterator<Item = &Command> {
        self.commands
            .values()
            .filter(|cmd| !cmd.disabled_by_default)
    }

    pub fn commands(&self) -> impl Iterator<Item = &Command> {
        self.commands.values()
    }
}
