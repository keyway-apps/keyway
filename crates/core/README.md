## Core

The `keyway_core` crate owns Keyway's shared command model and command registry.

The target command contract, including search-panel visibility, argument rules,
and generated CLI exposure, is defined in
[`docs/command-design.md`](../../docs/command-design.md).

Initialize core services before feature crates register commands:

```rust
app.run(move |cx| {
    keyway_core::init(cx);

    keyway_clipboard::init(cx);
});
```

```rust
use keyway_core::{Command, CommandRegistry};
use gpui::App;

pub fn init(cx: &mut App) {
    CommandRegistry::global(cx).update(cx, |registry, _cx| {
        registry.register_command(
            Command::new("example.open", "Open Example")
                .subtitle("Open an example command")
                .description("Shows how to register one command.")
                .keywords(["example", "open"]),
        );
    });
}
```

`CommandRegistry::register_commands` accepts any iterator of commands, and
`CommandProvider` can represent a named command source. Duplicate command IDs
are skipped and logged as warnings.
