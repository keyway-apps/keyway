use std::sync::Arc;

use aho_corasick::AhoCorasick;
use fancy_regex::Regex;
use gpui_kit::{AnyWeakEntity, App, Context, Entity, SharedString, Task, WeakEntity, Window};

use crate::item::{Item, WeakItemHandle};

#[derive(Clone, Debug)]
pub enum SearchQuery {
    Text {
        search: AhoCorasick,
        whole_word: bool,
        case_sensitive: bool,
        query: Arc<str>,
    },
    Regex {
        regex: Regex,
        whole_word: bool,
        case_sensitive: bool,
        query: Arc<str>,
    },
}

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

    fn find_matches(&mut self, query: Arc<SearchQuery>, window: &mut Window, cx: &mut Context<Self>) -> Task<()>;
}

pub trait SearchableItemHandle {
    fn downgrade(&self) -> Box<dyn WeakSearchableItemHandle>;
    fn boxed_clone(&self) -> Box<dyn SearchableItemHandle>;
    fn supported_options(&self, cx: &App) -> SearchOptions;
    fn placeholder_text(&self, cx: &App) -> Option<SharedString>;
    fn find_matches(&self, query: Arc<SearchQuery>, window: &mut Window, cx: &mut App) -> Task<()>;
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

    fn find_matches(&self, query: Arc<SearchQuery>, window: &mut Window, cx: &mut App) -> Task<()> {
        self.update(cx, |item, cx| item.find_matches(query, window, cx))
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
