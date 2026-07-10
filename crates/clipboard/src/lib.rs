use command::prelude::*;
use std::sync::LazyLock;

pub fn init() {
    command!(Clipboard);
}

pub struct Clipboard;

static COMMANDS: LazyLock<Vec<Command>> = LazyLock::new(|| {
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
});

impl CommandProvider for Clipboard {
    fn commands(&self) -> &[Command] {
        COMMANDS.as_slice()
    }
}