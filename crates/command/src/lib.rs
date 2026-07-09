mod command;

pub use command::*;

use gpui::{App, Global};
use collections::HashMap;

#[doc(hidden)]
pub use inventory::submit as __submit_command_provider;

pub mod prelude {
    pub use super::CommandProvider;
    pub use super::Command;
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
        inventory::iter::<&'static dyn CommandProvider>
            .into_iter()
            .flat_map(|provider| provider.commands().iter().cloned())
            .fold(
                Self {
                    commands: HashMap::default(),
                },
                |mut acc, command| {
                    acc.commands.insert(command.id.clone(), command);
                    acc
                },
            )
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.commands.values()
    }
}

impl Global for GlobalCommandRegistry {}
