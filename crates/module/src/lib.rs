use gpui::{App, AppContext, Entity, Global};

mod command;
mod inline;
mod module;
mod module_group;
mod shortcut;
mod view;

pub use command::*;
pub use module::*;
pub use module_group::*;

pub mod prelude {
    pub use super::{
        Actions, Command, CommandAction, CommandBuilder, CommandRegistry, Module, ModuleContext,
        ModuleGroup, ModuleGroupBuilder, ModuleStore,
    };
}

pub fn init(cx: &mut App) {
    let store = cx.new(ModuleStore::new);
    cx.set_global(GlobalModuleStore(store));
}

struct GlobalModuleStore(Entity<ModuleStore>);

impl Global for GlobalModuleStore {}
