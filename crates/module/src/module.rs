use anyhow::Result;
use core::any::Any;
use gpui::{AnyEntity, App, AppContext, Context, Entity, EntityId};
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
        let context = cx.new(ModuleContext::new);
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
            .update(cx, |context, cx: &mut Context<'_, ModuleContext>| {
                module.with_to_module(context, cx)
            });
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

struct ModuleSlot {
    module: Option<AnyModule>,
    ready: bool,
}

pub struct ModuleContext {
    entity_id: EntityId,
    module_registry: Vec<ModuleSlot>,
    module_names: HashSet<String>,
    pub(crate) command_registry: CommandRegistry,
}

impl ModuleContext {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            entity_id: cx.entity_id(),
            module_registry: Default::default(),
            module_names: Default::default(),
            command_registry: Default::default(),
        }
    }

    pub(crate) fn notify(&self, cx: &mut App) {
        cx.notify(self.entity_id);
    }

    pub(crate) fn with_boxed_module<M: Module>(
        &mut self,
        module: Entity<M>,
        group: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let module: AnyModule = module.into();
        
        let module_name = module.name(cx);
        if self.module_names.contains(&module_name) {
            tracing::warn!(
                "module-add-duplicate: {} {:?}",
                module_name,
                group.map(str::to_string)
            );
            return;
        }

        tracing::debug!("module-build-started: {module_name}");

        let build_result = catch_unwind(AssertUnwindSafe(|| module.build(self, cx)));

        match build_result {
            Ok(Ok(())) => {
                self.module_names.insert(module_name.clone());
                self.module_registry.push(ModuleSlot {
                    module: Some(module),
                    ready: false,
                });
                tracing::info!("module-build-finished: {module_name}");
            }
            Ok(Err(err)) => {
                tracing::error!("module-build-failed: {module_name} ({err:#})");
            }
            Err(panic) => {
                let detail = panic_message(&panic);
                tracing::error!("module-build-failed: {module_name} (panic: {detail})");
            }
        }
    }

    pub(crate) fn drive_ready(&mut self, cx: &mut Context<Self>) {
        tracing::debug!("module-ready-drive-started");
        let len = self.module_registry.len();
        for index in 0..len {
            let module = self.module_registry[index]
                .module
                .take()
                .expect("module is only absent while its lifecycle callback is running");
            let module_name = module.name(cx);
            tracing::debug!("module-ready-started: {module_name}");

            let result = catch_unwind(AssertUnwindSafe(|| module.ready(self, cx)));

            match result {
                Ok(Ok(())) => {
                    self.module_registry[index].ready = true;
                    tracing::info!("module-ready-finished: {module_name}");
                }
                Ok(Err(err)) => {
                    self.module_registry[index].ready = false;
                    tracing::error!("module-ready-failed: {module_name} ({err:#})");
                }
                Err(panic) => {
                    self.module_registry[index].ready = false;
                    let detail = panic_message(&panic);
                    tracing::error!("module-ready-failed: {module_name} (panic: {detail})");
                }
            }

            self.module_registry[index].module = Some(module);
        }
        tracing::debug!("module-ready-drive-finished");
    }

    pub(crate) fn drive_stop(&mut self, cx: &mut Context<Self>) {
        tracing::debug!("module-stop-drive-started");
        let len = self.module_registry.len();
        for index in (0..len).rev() {
            if !self.module_registry[index].ready {
                continue;
            }

            let module = self.module_registry[index]
                .module
                .take()
                .expect("module is only absent while its lifecycle callback is running");
            let module_name = module.name(cx);
            tracing::debug!("module-stop-started: {module_name}");

            let result = catch_unwind(AssertUnwindSafe(|| module.stop(self, cx)));

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

            self.module_registry[index].ready = false;
            self.module_registry[index].module = Some(module);
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

pub trait Module: 'static + Sized {
    fn build(&mut self, context: &mut ModuleContext, cx: &mut Context<Self>) -> Result<()>;

    fn ready(&mut self, _context: &mut ModuleContext, _cx: &mut Context<Self>) -> Result<()> {
        Ok(())
    }

    fn stop(&mut self, _context: &mut ModuleContext, _cx: &mut Context<Self>) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        core::any::type_name::<Self>()
    }
}

impl<T> Module for T
where
    T: Fn(&mut ModuleContext, &mut App) -> Result<()> + Send + Sync + 'static,
{
    fn build(&mut self, context: &mut ModuleContext, cx: &mut Context<Self>) -> Result<()> {
        self(context, cx)
    }
}

#[derive(Clone, Debug)]
pub struct AnyModule {
    entity: AnyEntity,
    name: fn(&AnyModule, &mut App) -> String,
    build: fn(&AnyModule, &mut ModuleContext, &mut App) -> Result<()>,
    ready: fn(&AnyModule, &mut ModuleContext, &mut App) -> Result<()>,
    stop: fn(&AnyModule, &mut ModuleContext, &mut App) -> Result<()>,
}

impl<M: Module> From<Entity<M>> for AnyModule {
    fn from(value: Entity<M>) -> Self {
        Self {
            entity: value.into_any(),
            name: any_module::name::<M>,
            build: any_module::build::<M>,
            ready: any_module::ready::<M>,
            stop: any_module::stop::<M>,
        }
    }
}

impl AnyModule {
    fn name(&self, cx: &mut App) -> String {
        (self.name)(self, cx)
    }

    fn build(&self, context: &mut ModuleContext, cx: &mut App) -> Result<()> {
        (self.build)(self, context, cx)
    }

    fn ready(&self, context: &mut ModuleContext, cx: &mut App) -> Result<()> {
        (self.ready)(self, context, cx)
    }

    fn stop(&self, context: &mut ModuleContext, cx: &mut App) -> Result<()> {
        (self.stop)(self, context, cx)
    }

    pub fn downcast<T: 'static>(self) -> Result<Entity<T>, Self> {
        match self.entity.downcast() {
            Ok(entity) => Ok(entity),
            Err(entity) => Err(Self {
                entity,
                name: self.name,
                build: self.build,
                ready: self.ready,
                stop: self.stop,
            }),
        }
    }
}

mod any_module {
    use crate::{AnyModule, Module, ModuleContext};
    use anyhow::Result;
    use gpui::App;

    pub(crate) fn name<V: 'static + Module>(module: &AnyModule, cx: &mut App) -> String {
        let module = module.clone().downcast::<V>().unwrap();
        module.read(cx).name().to_string()
    }

    pub(crate) fn build<V: 'static + Module>(
        module: &AnyModule,
        context: &mut ModuleContext,
        cx: &mut App,
    ) -> Result<()> {
        let module = module.clone().downcast::<V>().unwrap();
        module.update(cx, |module, cx| module.build(context, cx))
    }

    pub(crate) fn ready<V: 'static + Module>(
        module: &AnyModule,
        context: &mut ModuleContext,
        cx: &mut App,
    ) -> Result<()> {
        let module = module.clone().downcast::<V>().unwrap();
        module.update(cx, |module, cx| module.ready(context, cx))
    }

    pub(crate) fn stop<V: 'static + Module>(
        module: &AnyModule,
        context: &mut ModuleContext,
        cx: &mut App,
    ) -> Result<()> {
        let module = module.clone().downcast::<V>().unwrap();
        module.update(cx, |module, cx| module.stop(context, cx))
    }
}

pub trait Modules<Marker>: sealed::Modules<Marker> {}

impl<Marker, T> Modules<Marker> for T where T: sealed::Modules<Marker> {}

mod sealed {
    use gpui::{AppContext, Context};

    use super::{Module, ModuleContext};
    use crate::ModuleGroup;

    pub trait Modules<Marker> {
        fn with_to_module(self, context: &mut ModuleContext, cx: &mut Context<ModuleContext>);
    }

    pub struct ModuleMarker;
    pub struct ModuleGroupMarker;

    impl<M: Module> Modules<ModuleMarker> for M {
        fn with_to_module(self, context: &mut ModuleContext, cx: &mut Context<ModuleContext>) {
            context.with_boxed_module(cx.new(|_| self), None, cx);
        }
    }

    impl<M: ModuleGroup> Modules<ModuleGroupMarker> for M {
        fn with_to_module(self, context: &mut ModuleContext, cx: &mut Context<ModuleContext>) {
            self.build().finish(context, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };

    use crate::{Command, ModuleGroup, ModuleGroupBuilder, init};

    use super::*;

    struct LifecycleModule {
        events: Arc<Mutex<Vec<&'static str>>>,
    }

    impl Module for LifecycleModule {
        fn build(&mut self, context: &mut ModuleContext, cx: &mut Context<Self>) -> Result<()> {
            assert_ne!(context.entity_id, cx.entity_id());
            self.events.lock().unwrap().push("build");
            context.register_command(
                Command::new("test.lifecycle", "Lifecycle Test"),
                |_actions, _context, _cx| Ok(()),
                cx,
            );
            Ok(())
        }

        fn ready(&mut self, _context: &mut ModuleContext, _cx: &mut Context<Self>) -> Result<()> {
            self.events.lock().unwrap().push("ready");
            Ok(())
        }

        fn stop(&mut self, _context: &mut ModuleContext, _cx: &mut Context<Self>) -> Result<()> {
            self.events.lock().unwrap().push("stop");
            Ok(())
        }
    }

    struct LifecycleGroup {
        events: Arc<Mutex<Vec<&'static str>>>,
    }

    impl ModuleGroup for LifecycleGroup {
        fn build(self) -> ModuleGroupBuilder {
            ModuleGroupBuilder::start::<Self>().add(LifecycleModule {
                events: self.events,
            })
        }
    }

    #[gpui::test]
    async fn module_lifecycle_uses_the_module_entity_context(cx: &mut gpui::TestAppContext) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let closure_built = Arc::new(AtomicBool::new(false));

        cx.update(|cx| {
            init(cx);

            let store = ModuleStore::global(cx);
            store.update(cx, |store, cx| {
                store.with_modules(
                    LifecycleGroup {
                        events: events.clone(),
                    },
                    cx,
                );

                let closure_built = closure_built.clone();
                store.with_modules(
                    move |_context: &mut ModuleContext, _cx: &mut App| {
                        closure_built.store(true, Ordering::SeqCst);
                        Ok(())
                    },
                    cx,
                );

                store.drive_ready(cx);
                store.drive_stop(cx);
                store.drive_stop(cx);
            });

            let module_context = store.read(cx).context();
            assert_eq!(
                module_context
                    .read(cx)
                    .command_registry()
                    .commands()
                    .count(),
                1
            );
        });

        assert_eq!(&*events.lock().unwrap(), &["build", "ready", "stop"]);
        assert!(closure_built.load(Ordering::SeqCst));
    }
}
