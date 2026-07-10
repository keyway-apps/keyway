mod command;

pub use command::*;

use collections::HashMap;
use gpui::{App, Global};

#[doc(hidden)]
pub use inventory::submit as __submit_command_provider;

pub mod prelude {
    pub use super::Command;
    pub use super::CommandProvider;
    pub use super::command;
}

pub trait CommandProvider: Sync {
    fn commands(&self) -> &[Command];
}

#[macro_export]
macro_rules! command {
    ($provider:path $(,)?) => {
        const _: () = {
            fn _assert<T: $crate::CommandProvider>() {}
            let _ = _assert::<$provider>;
        };

        $crate::__submit_command_provider! {
            &$provider as &'static dyn $crate::CommandProvider
        }
    };
}

inventory::collect!(&'static dyn CommandProvider);

pub fn init(cx: &mut App) {
    cx.set_global(GlobalCommandRegistry::new());
}

pub struct GlobalCommandRegistry {
    commands: HashMap<String, Command>,
}

impl GlobalCommandRegistry {
    fn new() -> Self {
        let mut failed_count = 0;

        let commands = inventory::iter::<&'static dyn CommandProvider>
            .into_iter()
            .flat_map(|provider| provider.commands().iter().cloned())
            .fold(HashMap::default(), |mut acc, command| {
                if acc.contains_key(&command.id) {
                    failed_count += 1;
                } else {
                    acc.insert(command.id.clone(), command);
                }
                acc
            });

        let registered_count = commands.len();

        tracing::info!(
            registered_command_count = registered_count,
            "command registry initialized: commands inserted into the global command registry"
        );
        tracing::warn!(
            failed_command_count = failed_count,
            failure_reason = "duplicate command id",
            "command registry initialized: commands skipped because another command with the same id was already registered"
        );

        Self { commands }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.commands.values()
    }
}

impl Global for GlobalCommandRegistry {}
