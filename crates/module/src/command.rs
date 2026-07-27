use std::rc::Rc;

use collections::{HashMap, hash_map::Entry};
use gpui::Context;
use serde::{Deserialize, Serialize};

use crate::ModuleContext;

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

        if let Some(subtitle) = &self.subtitle {
            terms.push(subtitle);
        }
        if let Some(category) = &self.category {
            terms.push(category);
        }
        if let Some(description) = &self.description {
            terms.push(description);
        }
        if let Some(keywords) = &self.keywords {
            terms.extend(keywords.iter().map(String::as_str));
        }

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

    pub fn hidden_by_default(mut self) -> Self {
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
    Fn(&mut Actions, &mut ModuleContext, &mut Context<ModuleContext>) -> anyhow::Result<()> + 'static
{
}

impl<F> CommandAction for F where
    F: Fn(&mut Actions, &mut ModuleContext, &mut Context<ModuleContext>) -> anyhow::Result<()>
        + 'static
{
}

#[derive(Default)]
pub struct Actions {
    query: Option<String>,
}

impl Actions {
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = Some(query.into());
    }
}

#[derive(Default)]
pub struct CommandRegistry {
    commands: HashMap<String, Command>,
    actions: HashMap<String, Rc<dyn CommandAction>>,
}

impl CommandRegistry {
    fn register_command(&mut self, command: Command, action: impl CommandAction) -> bool {
        let command_id = command.id.clone();
        match self.commands.entry(command.id.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(command);
                self.actions.insert(command_id, Rc::new(action));
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

    fn unregister_commands<I, S>(&mut self, command_ids: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        command_ids.into_iter().fold(false, |removed, command_id| {
            self.remove_command(command_id.as_ref()) || removed
        })
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

    fn action(&self, command_id: &str) -> Option<Rc<dyn CommandAction>> {
        self.actions.get(command_id).cloned()
    }

    pub fn visible_commands(&self) -> impl Iterator<Item = &Command> {
        self.commands
            .values()
            .filter(|command| !command.disabled_by_default && command.visible_by_default)
    }

    pub fn enabled_commands(&self) -> impl Iterator<Item = &Command> {
        self.commands
            .values()
            .filter(|command| !command.disabled_by_default)
    }

    pub fn commands(&self) -> impl Iterator<Item = &Command> {
        self.commands.values()
    }
}

impl ModuleContext {
    pub fn command_registry(&self) -> &CommandRegistry {
        &self.command_registry
    }

    pub fn register_command(
        &mut self,
        command: Command,
        action: impl CommandAction,
        cx: &mut Context<Self>,
    ) {
        if self.command_registry.register_command(command, action) {
            cx.notify();
        }
    }

    pub fn unregister_command(&mut self, command_id: &str, cx: &mut Context<Self>) {
        if self.command_registry.remove_command(command_id) {
            cx.notify();
        }
    }

    pub fn unregister_commands<I, S>(&mut self, command_ids: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if self.command_registry.unregister_commands(command_ids) {
            cx.notify();
        }
    }

    pub fn execute_command(
        &mut self,
        command_id: &str,
        actions: &mut Actions,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let action = self
            .command_registry
            .action(command_id)
            .ok_or_else(|| anyhow::anyhow!("command not found: {command_id}"))?;
        action(actions, self, cx)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::*;
    use crate::{ModuleStore, init};

    #[gpui::test]
    async fn module_context_registers_and_executes_commands(cx: &mut gpui::TestAppContext) {
        let executed = Rc::new(Cell::new(false));
        let action_executed = executed.clone();

        cx.update(|cx| {
            init(cx);

            let store = ModuleStore::global(cx);
            let module_context = store.read(cx).context();

            module_context.update(cx, |context, cx| {
                context.register_command(
                    Command::new("test.command", "Test Command"),
                    move |actions, context, _cx| {
                        actions.set_query("executed");
                        assert_eq!(context.command_registry().commands().count(), 1);
                        action_executed.set(true);
                        Ok(())
                    },
                    cx,
                );

                let mut actions = Actions::default();
                context
                    .execute_command("test.command", &mut actions, cx)
                    .unwrap();

                assert_eq!(actions.query(), Some("executed"));
                assert_eq!(context.command_registry().visible_commands().count(), 1);
            });
        });

        assert!(executed.get());
    }
}
