use gpui::{App, Context};
use keyway_core::prelude::*;

pub fn init(cx: &mut App) {
    CommandRegistry::global(cx).update(cx, |registry, cx| {
        registry.register_provider(Clipboard, cx);
    });
}

pub struct Clipboard;

impl CommandProvider for Clipboard {
    type Commands = Vec<Command>;

    fn commands(&self, _cx: &mut Context<CommandRegistry>) -> Self::Commands {
        vec![
            Command::new("clipboard.history", "Clipboard History")
                .subtitle("Search copied text, links, and snippets")
                .description("Open clipboard history and select an item to paste.")
                .keywords(["copy", "paste", "history"]),
            Command::new("clipboard.clear_history", "Clear Clipboard History")
                .subtitle("Remove saved clipboard entries")
                .description("Clear all locally stored clipboard history entries.")
                .keywords(["clear", "delete", "privacy"]),
            Command::new("clipboard.pin_current", "Pin Current Clipboard")
                .subtitle("Keep the current clipboard item")
                .description("Pin the current clipboard item so it remains available.")
                .keywords(["pin", "favorite", "save"]),
        ]
    }
}
