use anyhow::Result;
use gpui::{App, Context};
use module::prelude::*;

pub fn init(cx: &mut App) {
    ModuleStore::global(cx).update(cx, |store, cx| {
        store.add::<ClipboardModule>(cx);
    });
}

#[derive(Default)]
pub struct ClipboardModule;

impl Module for ClipboardModule {
    fn build(&self, context: &mut ModuleContext, cx: &mut Context<ModuleContext>) -> Result<()> {
        let command = CommandBuilder::new("clipboard.history", "Clipboard History")
            .description("Open clipboard history and select an item to paste.")
            .keywords(["copy", "history"])
            .build();

        context.register_command(command, |_actions, _context, _cx| Ok(()), cx);

        Ok(())
    }
}
