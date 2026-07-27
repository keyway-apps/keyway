`ModuleContext` owns one scoped `CommandRegistry`. Module code mutates commands
through `ModuleContext` so GPUI observers are notified, while consumers can use
`ModuleContext::command_registry` for read-only command queries.

```rust
use gpui::{App, Context};
use module::prelude::*;

pub fn init(cx: &mut App) {
    ModuleStore::global(cx).update(cx, |store, cx| {
        store.add::<ClipboardModule>(cx);
    });
}

pub struct ClipboardView {}

impl ViewRender for ClipboardView {
    fn render(&self, context: &mut ViewContext, cx: &mut Context<Self>) -> impl IntoElement {
        context
            .query
            .set_placeholder("Search your clipboard history...", cx);

        div()
    }
}

pub struct ClipboardInline;

impl InlineProvider for ClipboardInline {
    fn score(&self, query: &str) -> Option<u16> {
        Some(1000)
    }

    fn render(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Default)]
pub struct ClipboardModule;

impl Module for ClipboardModule {
    fn build(
        &self,
        mc: &mut ModuleContext,
        cx: &mut Context<ModuleContext>,
    ) -> anyhow::Result<()> {
        let command = CommandBuilder::new("clipboard.history", "Clipboard History")
            .description("Open clipboard history and select an item to paste.")
            .keywords(["copy", "history"])
            .shortcut("Ctrl+R")
            .build();

        mc.register_command(
            command,
            |_, mc, cx| {
                mc.open_view(options, |window, cx| {
                    cx.new(|cx| ClipboardView::new(window, cx))
                });

                Ok(())
            },
            cx,
        );

        // 只是注册快捷键，把注册的快捷键放在设置中，当用户修改注册的快捷键，产生回调
        let shortcut = Shortcut::new("Alt+R");
        mc.register_shortcut(shortcut, |shortcut, cx| {
            // 用户修改了快捷键
        })

        let inline = cx.new(|cx| ClipboardInline::new(cx)); 
        mc.register_inline(inline, |_, mc, cx| {
            mc.open_view(options, |window, cx| {
                cx.new(|cx| ClipboardView::new(window, cx))
            })

            Ok(())
        }, cx);

        Ok(())
    }
}

```
