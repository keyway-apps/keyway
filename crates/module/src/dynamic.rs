use std::sync::Arc;

use anyhow::Result;
use gpui::{AnyElement, AnyEntity, App, Context, Entity, IntoElement, Window};
use regex::RegexSet;

use collections::{HashMap, hash_map::Entry};

use crate::ModuleContext;

/// Static metadata and query pre-matching rules for a dynamic result provider.
#[derive(Debug)]
pub struct Dynamic {
    id: String,
    title: String,
    match_regexes: Option<RegexSet>,
}

impl Dynamic {
    pub fn builder(id: impl Into<String>, title: impl Into<String>) -> DynamicBuilder {
        DynamicBuilder::new(id, title)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn match_regexes(&self) -> &[String] {
        self.match_regexes
            .as_ref()
            .map(RegexSet::patterns)
            .unwrap_or_default()
    }

    /// Returns whether this provider should perform its more specific scoring.
    ///
    /// Providers without regular expressions receive every query. Multiple
    /// expressions use OR semantics.
    pub fn matches_query(&self, query: &str) -> bool {
        self.match_regexes
            .as_ref()
            .is_none_or(|regexes| regexes.is_match(query))
    }
}

#[derive(Clone, Debug)]
pub struct DynamicBuilder {
    id: String,
    title: String,
    match_regexes: Vec<String>,
}

impl DynamicBuilder {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            match_regexes: Vec::new(),
        }
    }

    pub fn match_regex(mut self, regex: impl Into<String>) -> Self {
        self.match_regexes.push(regex.into());
        self
    }

    pub fn match_regexes<I, R>(mut self, regexes: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: Into<String>,
    {
        self.match_regexes
            .extend(regexes.into_iter().map(Into::into));
        self
    }

    pub fn build(self) -> std::result::Result<Dynamic, regex::Error> {
        let match_regexes = if self.match_regexes.is_empty() {
            None
        } else {
            Some(RegexSet::new(self.match_regexes)?)
        };

        Ok(Dynamic {
            id: self.id,
            title: self.title,
            match_regexes,
        })
    }
}

/// A provider for results that are computed from the current launcher query.
pub trait DynamicRender: 'static + Sized {
    fn score(&self, query: &str, cx: &Context<Self>) -> Option<u16>;

    fn render(&self, window: &mut Window, cx: &Context<Self>) -> impl IntoElement;

    fn activate(&self, context: &mut ModuleContext, cx: &mut Context<Self>) -> Result<()>;
}

/// A type-erased handle to a [`DynamicRender`] GPUI entity and its metadata.
#[derive(Clone, Debug)]
pub struct AnyDynamic {
    dynamic: Arc<Dynamic>,
    entity: AnyEntity,
    score: fn(&AnyDynamic, &str, &mut App) -> Option<u16>,
    render: fn(&AnyDynamic, window: &mut Window, &mut App) -> AnyElement,
    activate: fn(&AnyDynamic, &mut ModuleContext, &mut App) -> Result<()>,
}

impl AnyDynamic {
    fn new<D: DynamicRender>(dynamic: Dynamic, entity: Entity<D>) -> Self {
        Self {
            dynamic: Arc::new(dynamic),
            entity: entity.into_any(),
            score: any_dynamic::score::<D>,
            render: any_dynamic::render::<D>,
            activate: any_dynamic::activate::<D>,
        }
    }

    pub fn id(&self) -> &str {
        self.dynamic.id()
    }

    pub fn title(&self) -> &str {
        self.dynamic.title()
    }

    pub fn match_regexes(&self) -> &[String] {
        self.dynamic.match_regexes()
    }

    pub fn matches_query(&self, query: &str) -> bool {
        self.dynamic.matches_query(query)
    }

    pub fn score(&self, query: &str, cx: &mut App) -> Option<u16> {
        if !self.matches_query(query) {
            return None;
        }

        (self.score)(self, query, cx)
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        (self.render)(self, window, cx)
    }

    pub fn activate(&self, context: &mut ModuleContext, cx: &mut App) -> Result<()> {
        (self.activate)(self, context, cx)
    }

    pub fn downcast<T: 'static>(self) -> Result<Entity<T>, Self> {
        match self.entity.downcast() {
            Ok(entity) => Ok(entity),
            Err(entity) => Err(Self {
                dynamic: self.dynamic,
                entity,
                score: self.score,
                render: self.render,
                activate: self.activate,
            }),
        }
    }
}

mod any_dynamic {
    use super::{AnyDynamic, DynamicRender};
    use anyhow::Result;
    use gpui::{App, IntoElement, Window};

    pub(crate) fn score<D: DynamicRender>(
        dynamic: &AnyDynamic,
        query: &str,
        cx: &mut App,
    ) -> Option<u16> {
        let dynamic = dynamic
            .clone()
            .downcast::<D>()
            .expect("dynamic entity type mismatch");
        dynamic.update(cx, |dynamic, cx| dynamic.score(query, cx))
    }

    pub(crate) fn render<D: DynamicRender>(
        dynamic: &AnyDynamic,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyElement {
        let dynamic = dynamic
            .clone()
            .downcast::<D>()
            .expect("dynamic entity type mismatch");
        dynamic.update(cx, |dynamic, cx| {
            dynamic.render(window, cx).into_any_element()
        })
    }

    pub(crate) fn activate<D: DynamicRender>(
        dynamic: &AnyDynamic,
        context: &mut super::ModuleContext,
        cx: &mut App,
    ) -> Result<()> {
        let dynamic = dynamic
            .clone()
            .downcast::<D>()
            .expect("dynamic entity type mismatch");
        dynamic.update(cx, |dynamic, cx| dynamic.activate(context, cx))
    }
}

#[derive(Default)]
pub(crate) struct DynamicRegistry {
    dynamics: HashMap<String, AnyDynamic>,
}

impl DynamicRegistry {
    fn contains_dynamic(&self, dynamic_id: &str) -> bool {
        self.dynamics.contains_key(dynamic_id)
    }

    fn register_dynamic(&mut self, dynamic: AnyDynamic) -> bool {
        let dynamic_id = dynamic.id().to_owned();
        match self.dynamics.entry(dynamic_id.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(dynamic);
                true
            }
            Entry::Occupied(_) => {
                tracing::warn!(
                    dynamic_id,
                    "duplicate dynamic registration detected, skipping"
                );
                false
            }
        }
    }

    fn unregister_dynamics<I, S>(&mut self, dynamic_ids: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut removed = false;
        for dynamic_id in dynamic_ids {
            removed |= self.remove_dynamic(dynamic_id.as_ref());
        }
        removed
    }

    fn remove_dynamic(&mut self, dynamic_id: &str) -> bool {
        if self.dynamics.remove(dynamic_id).is_some() {
            true
        } else {
            tracing::warn!(
                dynamic_id,
                "dynamic unregistration failed, dynamic not found"
            );
            false
        }
    }

    fn dynamic(&self, dynamic_id: &str) -> Option<&AnyDynamic> {
        self.dynamics.get(dynamic_id)
    }

    fn dynamics(&self) -> impl Iterator<Item = &AnyDynamic> {
        self.dynamics.values()
    }
}

impl ModuleContext {
    pub fn dynamic(&self, dynamic_id: &str) -> Option<&AnyDynamic> {
        self.dynamic_registry.dynamic(dynamic_id)
    }

    pub fn dynamics(&self) -> impl Iterator<Item = &AnyDynamic> {
        self.dynamic_registry.dynamics()
    }

    pub fn register_dynamic<D, F>(&mut self, dynamic: Dynamic, build: F, cx: &mut App)
    where
        D: DynamicRender,
        F: FnOnce(&mut ModuleContext, &mut App) -> Entity<D>,
    {
        if self.dynamic_registry.contains_dynamic(dynamic.id()) {
            tracing::warn!(
                dynamic_id = dynamic.id(),
                "duplicate dynamic registration detected, skipping"
            );
            return;
        }

        let entity = build(self, cx);
        if self
            .dynamic_registry
            .register_dynamic(AnyDynamic::new(dynamic, entity))
        {
            self.notify(cx);
        }
    }

    pub fn unregister_dynamic(&mut self, dynamic_id: &str, cx: &mut App) {
        if self.dynamic_registry.remove_dynamic(dynamic_id) {
            self.notify(cx);
        }
    }

    pub fn unregister_dynamics<I, S>(&mut self, dynamic_ids: I, cx: &mut App)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if self.dynamic_registry.unregister_dynamics(dynamic_ids) {
            self.notify(cx);
        }
    }

    pub fn activate_dynamic(
        &mut self,
        dynamic_id: &str,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let dynamic = self
            .dynamic(dynamic_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("dynamic not found: {dynamic_id}"))?;
        dynamic.activate(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{AppContext, Context, Empty, IntoElement, Window};

    use super::*;
    use crate::{ModuleStore, init};

    struct TestDynamic {
        score_calls: Rc<Cell<usize>>,
        activated: Rc<Cell<bool>>,
    }

    impl DynamicRender for TestDynamic {
        fn score(&self, query: &str, _cx: &Context<Self>) -> Option<u16> {
            self.score_calls.set(self.score_calls.get() + 1);
            (query == "match").then_some(1_000)
        }

        fn render(&self, _window: &mut Window, _cx: &Context<Self>) -> impl IntoElement {
            Empty
        }

        fn activate(
            &self,
            _context: &mut ModuleContext,
            _cx: &mut Context<Self>,
        ) -> anyhow::Result<()> {
            self.activated.set(true);
            Ok(())
        }
    }

    #[test]
    fn dynamic_builder_compiles_query_pre_matchers() {
        let dynamic = DynamicBuilder::new("calculator.result", "Calculator")
            .match_regexes([r"^\d+\s*[+*/-]\s*\d+$", r"^sqrt\("])
            .build()
            .unwrap();

        assert_eq!(dynamic.id(), "calculator.result");
        assert_eq!(dynamic.title(), "Calculator");
        assert_eq!(dynamic.match_regexes().len(), 2);
        assert!(dynamic.matches_query("1 + 2"));
        assert!(dynamic.matches_query("sqrt(9)"));
        assert!(!dynamic.matches_query("clipboard"));

        let unfiltered = DynamicBuilder::new("test.all", "All").build().unwrap();
        assert!(unfiltered.matches_query("any query"));
    }

    #[test]
    fn dynamic_builder_rejects_invalid_regular_expressions() {
        assert!(
            DynamicBuilder::new("test.invalid", "Invalid")
                .match_regex("(")
                .build()
                .is_err()
        );
    }

    #[gpui::test]
    async fn module_context_builds_and_activates_dynamic_providers(cx: &mut gpui::TestAppContext) {
        let score_calls = Rc::new(Cell::new(0));
        let activated = Rc::new(Cell::new(false));

        cx.update(|cx| {
            init(cx);

            let module_context = ModuleStore::global(cx).read(cx).context();
            module_context.update(cx, |context, cx| {
                let dynamic = DynamicBuilder::new("test.dynamic", "Test Dynamic")
                    .match_regex("^match$")
                    .build()
                    .unwrap();
                let provider_score_calls = score_calls.clone();
                let activated = activated.clone();
                context.register_dynamic(
                    dynamic,
                    move |_context, cx| {
                        cx.new(|_| TestDynamic {
                            score_calls: provider_score_calls,
                            activated,
                        })
                    },
                    cx,
                );

                assert_eq!(context.dynamics().count(), 1);
                let registered = context.dynamic("test.dynamic").cloned().unwrap();
                assert_eq!(registered.id(), "test.dynamic");
                assert_eq!(registered.title(), "Test Dynamic");
                assert!(registered.clone().downcast::<TestDynamic>().is_ok());
                assert_eq!(registered.score("match", cx), Some(1_000));
                assert_eq!(registered.score("miss", cx), None);
                assert_eq!(score_calls.get(), 1);

                context.activate_dynamic("test.dynamic", cx).unwrap();
                context.unregister_dynamic("test.dynamic", cx);
                assert!(context.dynamic("test.dynamic").is_none());
            });
        });

        assert_eq!(score_calls.get(), 1);
        assert!(activated.get());
    }
}
