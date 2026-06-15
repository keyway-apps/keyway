pub use rust_i18n::{i18n, t};

pub fn init(locale: &str) {
    rust_i18n::set_locale(locale);
}
