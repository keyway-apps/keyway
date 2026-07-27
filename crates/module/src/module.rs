use anyhow::Result;
use core::any::Any;
use gpui::{App, AppContext, Context, Entity};
use std::panic::{AssertUnwindSafe, catch_unwind};

use collections::HashSet;

use crate::{CommandRegistry, GlobalModuleStore};

pub struct ModuleStore {
    context: Entity<ModuleContext>,
}

impl ModuleStore {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalModuleStore>().0.clone()
    }

    pub fn new(cx: &mut Context<Self>) -> Self {
        let context = cx.new(|_| ModuleContext::new());
        Self { context }
    }

    pub fn context(&self) -> Entity<ModuleContext> {
        self.context.clone()
    }

    pub fn add<M>(&mut self, cx: &mut Context<Self>)
    where
        M: Module + Default,
    {
        self.with_modules(M::default(), cx);
    }

    pub fn with_modules<M>(&mut self, module: impl Modules<M>, cx: &mut Context<Self>) {
        self.context
            .update(cx, |context, cx| module.with_to_module(context, cx));
    }

    pub fn drive_ready(&mut self, cx: &mut Context<Self>) {
        self.context
            .update(cx, |context, cx| context.drive_ready(cx));
    }

    pub fn drive_stop(&mut self, cx: &mut Context<Self>) {
        self.context
            .update(cx, |context, cx| context.drive_stop(cx));
    }
}

pub(crate) struct ModuleSlot {
    pub(crate) module: Box<dyn Module>,
    pub(crate) built: bool,
    pub(crate) ready: bool,
}

pub struct ModuleContext {
    pub(crate) module_registry: Vec<ModuleSlot>,
    pub(crate) module_names: HashSet<String>,
    pub(crate) command_registry: CommandRegistry,
}

impl ModuleContext {
    fn new() -> Self {
        Self {
            module_registry: Default::default(),
            module_names: Default::default(),
            command_registry: Default::default(),
        }
    }

    pub(crate) fn with_boxed_module(
        &mut self,
        module: Box<dyn Module>,
        group: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let module_name = module.name().to_string();
        if self.module_names.contains(&module_name) {
            tracing::warn!(
                "module-add-duplicate: {} {:?}",
                module_name,
                group.map(str::to_string)
            );
            return;
        }

        let index = self.module_registry.len();
        self.module_registry.push(ModuleSlot {
            module: Box::new(PlaceholderModule),
            built: false,
            ready: false,
        });

        tracing::debug!("module-build-started: {module_name}");

        let build_result = catch_unwind(AssertUnwindSafe(|| module.build(self, cx)));

        match build_result {
            Ok(Ok(())) => {
                self.module_names.insert(module_name);
                self.module_registry[index] = ModuleSlot {
                    module,
                    built: true,
                    ready: false,
                };
                tracing::info!(
                    "module-build-finished: {}",
                    self.module_registry[index].module.name()
                );
            }
            Ok(Err(err)) => {
                self.module_registry.remove(index);
                tracing::error!("module-build-failed: {module_name} ({err:#})");
            }
            Err(panic) => {
                self.module_registry.remove(index);
                let detail = panic_message(&panic);
                tracing::error!("module-build-failed: {module_name} (panic: {detail})");
            }
        }
    }

    pub(crate) fn drive_ready(&mut self, cx: &mut Context<Self>) {
        tracing::debug!("module-ready-drive-started");
        let len = self.module_registry.len();
        for index in 0..len {
            let mut slot = std::mem::replace(
                &mut self.module_registry[index],
                ModuleSlot {
                    module: Box::new(PlaceholderModule),
                    built: false,
                    ready: false,
                },
            );

            if !slot.built {
                self.module_registry[index] = slot;
                continue;
            }

            let module_name = slot.module.name().to_string();
            tracing::debug!("module-ready-started: {module_name}");
            let result = catch_unwind(AssertUnwindSafe(|| slot.module.ready(self, cx)));

            match result {
                Ok(Ok(())) => {
                    slot.ready = true;
                    tracing::info!("module-ready-finished: {module_name}");
                }
                Ok(Err(err)) => {
                    slot.ready = false;
                    tracing::error!("module-ready-failed: {module_name} ({err:#})");
                }
                Err(panic) => {
                    slot.ready = false;
                    let detail = panic_message(&panic);
                    tracing::error!("module-ready-failed: {module_name} (panic: {detail})");
                }
            }

            self.module_registry[index] = slot;
        }
        tracing::debug!("module-ready-drive-finished");
    }

    pub(crate) fn drive_stop(&mut self, cx: &mut Context<Self>) {
        tracing::debug!("module-stop-drive-started");
        let len = self.module_registry.len();
        for index in (0..len).rev() {
            let mut slot = std::mem::replace(
                &mut self.module_registry[index],
                ModuleSlot {
                    module: Box::new(PlaceholderModule),
                    built: false,
                    ready: false,
                },
            );

            if !slot.ready {
                self.module_registry[index] = slot;
                continue;
            }

            let module_name = slot.module.name().to_string();
            tracing::debug!("module-stop-started: {module_name}");
            let result = catch_unwind(AssertUnwindSafe(|| slot.module.stop(self, cx)));

            match result {
                Ok(Ok(())) => {
                    tracing::info!("module-stop-finished: {module_name}");
                }
                Ok(Err(err)) => {
                    tracing::error!("module-stop-failed: {module_name} ({err:#})");
                }
                Err(panic) => {
                    let detail = panic_message(&panic);
                    tracing::error!("module-stop-failed: {module_name} (panic: {detail})");
                }
            }

            // ready 已消费，置回 false 防止重复 stop。
            slot.ready = false;
            self.module_registry[index] = slot;
        }
        tracing::debug!("module-stop-drive-finished");
    }
}

fn panic_message(payload: &Box<dyn Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<unknown panic payload>".to_string()
    }
}

pub trait Module: Any + Send + Sync {
    fn build(&self, context: &mut ModuleContext, cx: &mut Context<ModuleContext>) -> Result<()>;

    fn ready(&self, _context: &mut ModuleContext, _cx: &mut Context<ModuleContext>) -> Result<()> {
        Ok(())
    }

    fn stop(&self, _context: &mut ModuleContext, _cx: &mut Context<ModuleContext>) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        core::any::type_name::<Self>()
    }
}

impl<T: Fn(&mut ModuleContext, &mut Context<ModuleContext>) -> Result<()> + Send + Sync + 'static>
    Module for T
{
    fn build(&self, context: &mut ModuleContext, cx: &mut Context<ModuleContext>) -> Result<()> {
        self(context, cx)
    }
}

pub(crate) struct PlaceholderModule;

impl Module for PlaceholderModule {
    fn build(&self, _context: &mut ModuleContext, _cx: &mut Context<ModuleContext>) -> Result<()> {
        Ok(())
    }
}

pub trait Modules<Marker>: sealed::Modules<Marker> {}

impl<Marker, T> Modules<Marker> for T where T: sealed::Modules<Marker> {}

mod sealed {
    use gpui::Context;

    use super::{Module, ModuleContext};
    use crate::ModuleGroup;

    pub trait Modules<Marker> {
        fn with_to_module(self, context: &mut ModuleContext, cx: &mut Context<ModuleContext>);
    }

    pub struct ModuleMarker;
    pub struct ModuleGroupMarker;

    impl<M: Module> Modules<ModuleMarker> for M {
        fn with_to_module(self, context: &mut ModuleContext, cx: &mut Context<ModuleContext>) {
            context.with_boxed_module(Box::new(self), None, cx);
        }
    }

    impl<M: ModuleGroup> Modules<ModuleGroupMarker> for M {
        fn with_to_module(self, context: &mut ModuleContext, cx: &mut Context<ModuleContext>) {
            self.build().finish(context, cx);
        }
    }
}
