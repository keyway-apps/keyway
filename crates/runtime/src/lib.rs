mod command;
mod registry;

pub use command::*;
pub use registry::*;

use gpui::{App, AppContext, Entity, Global};

pub mod prelude {
    pub use super::{Command, CommandProvider, CommandRegistry};
}

pub fn init(cx: &mut App) {
    let registry = cx.new(|_cx| CommandRegistry::new());
    cx.set_global(GlobalCommandRegistry(registry));
}

struct GlobalCommandRegistry(Entity<CommandRegistry>);

impl Global for GlobalCommandRegistry {}
