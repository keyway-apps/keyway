use core::any::TypeId;
use gpui::{AppContext, Context};

use util::TypeIdMap;

use crate::{Module, ModuleContext};

pub trait ModuleGroup {
    fn build(self) -> ModuleGroupBuilder;

    fn name() -> String {
        core::any::type_name::<Self>().to_string()
    }
}

struct ModuleEntry {
    // module: Box<dyn Module>,
    enabled: bool,
}

#[derive(Default)]
pub struct ModuleGroupBuilder {
    group_name: String,
    modules: TypeIdMap<ModuleEntry>,
    order: Vec<TypeId>,
}

impl ModuleGroupBuilder {
    pub fn start<MG: ModuleGroup>() -> Self {
        Self {
            group_name: MG::name(),
            modules: Default::default(),
            order: Default::default(),
        }
    }

    // fn upsert_module_state<T: Module>(&mut self, module: T, added_at_index: usize) {
    //     self.upsert_module_entry_state(
    //         TypeId::of::<T>(),
    //         ModuleEntry {
    //             module: Box::new(module),
    //             enabled: true,
    //         },
    //         added_at_index,
    //     );
    // }

    // fn upsert_module_entry_state(
    //     &mut self,
    //     key: TypeId,
    //     module: ModuleEntry,
    //     added_at_index: usize,
    // ) {
    //     if let Some(entry) = self.modules.insert(key, module) {
    //         if entry.enabled {
    //             panic!(
    //                 "You are replacing module '{}' that was not disabled.",
    //                 entry.module.name()
    //             );
    //         }
    //         if let Some(to_remove) = self
    //             .order
    //             .iter()
    //             .enumerate()
    //             .find(|(i, ty)| *i != added_at_index && **ty == key)
    //             .map(|(i, _)| i)
    //         {
    //             self.order.remove(to_remove);
    //         }
    //     }
    // }

    pub fn add<T: Module>(mut self, module: T) -> Self {
        // let target_index = self.order.len();
        // self.order.push(TypeId::of::<T>());
        // self.upsert_module_state(module, target_index);
        self
    }

    pub fn finish(mut self, context: &mut ModuleContext, cx: &mut Context<ModuleContext>) {
        // for ty in &self.order {
        //     if let Some(entry) = self.modules.remove(ty)
        //         && entry.enabled
        //     {
        //         let module = cx.new(|_| entry.module );
        //         context.with_boxed_module(module, Some(&self.group_name), cx);
        //     }
        // }
    }
}
