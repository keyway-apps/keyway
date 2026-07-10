use collections::{HashMap, hash_map::Entry};
use gpui::{App, Entity};

use crate::{Command,GlobalCommandRegistry};

pub trait CommandProvider {
    type Commands: IntoIterator<Item = Command>;
    fn commands(&self) -> Self::Commands;
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

    pub fn register_provider<T: CommandProvider>(&mut self, provider: T) {
        self.register_commands(provider.commands());
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
            Entry::Vacant(e) => {
                e.insert(command);
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
