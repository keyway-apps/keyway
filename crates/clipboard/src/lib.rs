use gpui::App;
use keyway_core::prelude::*;

pub fn init(cx: &mut App) {
    Clipboard::new(cx);
}

pub struct Clipboard;

impl Clipboard {
    fn new(cx: &mut App) {
        let command = CommandBuilder::new("clipboard.history", "Clipboard History")
            .description("Open clipboard history and select an item to paste.")
            .keywords(["copy", "history"])
            .build();

        CommandRegistry::global(cx).update(cx, |registry, cx| {
            registry.register_command(cx, command, |_, _| Ok(()));
        });
    }
}
