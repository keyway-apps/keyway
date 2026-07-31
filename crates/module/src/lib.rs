use gpui::{App, AppContext, Entity, Global};

mod command;
mod dynamic;
mod module;
mod module_group;
mod shortcut;
mod view;

pub use command::*;
pub use dynamic::*;
pub use module::*;
pub use module_group::*;
pub use ui::Icon;
pub use view::*;

pub mod prelude {
    pub use super::{
        Actions, AnyDynamic, Command, CommandAction, CommandBuilder, DynamicRender, Icon, Module,
        ModuleContext, ModuleGroup, ModuleGroupBuilder, ModuleStore,
    };
}

pub fn init(cx: &mut App) {
    let store = cx.new(ModuleStore::new);
    cx.set_global(GlobalModuleStore(store));
}

struct GlobalModuleStore(Entity<ModuleStore>);

impl Global for GlobalModuleStore {}
