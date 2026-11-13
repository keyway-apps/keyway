use gpui_kit::{AnyWeakEntity, App, Entity, SharedString, WeakEntity};

use crate::item::{Item, WeakItemHandle};

#[derive(Clone, Copy, Debug, Default)]
pub struct SearchOptions {
    pub case: bool,
    pub word: bool,
    pub regex: bool,
}

pub trait SearchableItem: Item {
    fn supported_options(&self) -> SearchOptions {
        SearchOptions {
            case: false,
            word: false,
            regex: false,
        }
    }

    fn placeholder_text(&self, _cx: &App) -> Option<SharedString> {
        None
    }
}

pub trait SearchableItemHandle {
    fn downgrade(&self) -> Box<dyn WeakSearchableItemHandle>;
    fn boxed_clone(&self) -> Box<dyn SearchableItemHandle>;
    fn supported_options(&self, cx: &App) -> SearchOptions;
    fn placeholder_text(&self, cx: &App) -> Option<SharedString>;
}

impl<T: SearchableItem> SearchableItemHandle for Entity<T> {
    fn downgrade(&self) -> Box<dyn WeakSearchableItemHandle> {
        Box::new(self.downgrade())
    }

    fn boxed_clone(&self) -> Box<dyn SearchableItemHandle> {
        Box::new(self.clone())
    }

    fn supported_options(&self, cx: &App) -> SearchOptions {
        self.read(cx).supported_options()
    }

    fn placeholder_text(&self, cx: &App) -> Option<SharedString> {
        self.read(cx).placeholder_text(cx)
    }
}

pub trait WeakSearchableItemHandle: WeakItemHandle {
    fn upgrade(&self, cx: &App) -> Option<Box<dyn SearchableItemHandle>>;
    fn into_any(self) -> AnyWeakEntity;
}

impl<T: SearchableItem> WeakSearchableItemHandle for WeakEntity<T> {
    fn upgrade(&self, _cx: &App) -> Option<Box<dyn SearchableItemHandle>> {
        Some(Box::new(self.upgrade()?))
    }

    fn into_any(self) -> AnyWeakEntity {
        self.into()
    }
}
