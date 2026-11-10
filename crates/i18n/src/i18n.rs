//! Shared locale selection. Translation catalogs belong to the crates that use them.
//!
//! Initialize the locale before UI components or windows. Application language changes
//! require a restart; this crate does not notify existing views or update cached text.

pub use rust_i18n::{locale, set_locale};

mod language;

#[doc(hidden)]
pub mod __private {
    pub use gpui_kit::SharedString;
    pub use rust_i18n::t;
}

/// Translate using the calling crate's catalog into a [`gpui_kit::SharedString`].
///
/// Declare the catalog with `rust_i18n::i18n!` in the calling crate's root. Arguments
/// follow `rust_i18n::t!`, including borrowed keys, interpolation and explicit locales.
#[macro_export]
macro_rules! tr {
    ($($args:tt)*) => {
        $crate::__private::SharedString::from($crate::__private::t!($($args)*))
    };
}

/// Select the application language before initializing UI components or windows.
pub fn init() {
    let system_locales: Vec<_> = sys_locale::get_locales().collect();
    let locale = language::system_locale(system_locales.iter().map(String::as_str));

    tracing::info!(locale, "Initialized internationalization");
    tracing::debug!(?system_locales, locale, "Resolved system locale preferences");

    set_locale(locale);
}
