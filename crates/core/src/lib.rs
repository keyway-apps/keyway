mod command;

pub use command::*;

use gpui::{App, AppContext, Entity, Global};

pub mod prelude {
    pub use super::{Command, CommandBuilder, CommandRegistry};
}

pub fn init(cx: &mut App) {
    let registry = cx.new(|_cx| CommandRegistry::new());
    cx.set_global(GlobalCommandRegistry(registry));
}

struct GlobalCommandRegistry(Entity<CommandRegistry>);

impl Global for GlobalCommandRegistry {}
