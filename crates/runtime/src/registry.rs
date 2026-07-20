use collections::{HashMap, hash_map::Entry};
use gpui::{App, Context, Entity};

use crate::{Command, GlobalCommandRegistry};

pub trait CommandProvider {
    type Commands: IntoIterator<Item = Command>;
    fn commands(&self, cx: &mut Context<CommandRegistry>) -> Self::Commands;
}

pub struct CommandRegistry {
    commands: HashMap<String, Command>,
}

impl CommandRegistry {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalCommandRegistry>().0.clone()
    }

    pub(crate) fn new() -> Self {
        Self {
            commands: HashMap::default(),
        }
    }

    pub fn register_provider<T: CommandProvider>(&mut self, provider: T, cx: &mut Context<Self>) {
        self.register_commands(provider.commands(cx));
    }

    pub fn register_command(&mut self, command: Command) {
        self.insert_command(command);
    }

    pub fn register_commands<T: IntoIterator<Item = Command>>(&mut self, commands: T) {
        commands
            .into_iter()
            .for_each(|command| self.insert_command(command));
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.commands.values()
    }

    fn insert_command(&mut self, command: Command) {
        match self.commands.entry(command.id.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(command);
            }
            Entry::Occupied(_) => {
                tracing::warn!(
                    command_id = %command.id,
                    "duplicate command registration detected, skipping"
                );
            }
        };
    }
}
