## Command

`command` crate owns Keyway command metadata and the in-process command registry.

The target domain contract, including search-panel visibility, argument rules, and generated CLI exposure, is defined in [`docs/command-design.md`](../../docs/command-design.md).

The application must initialize the registry before feature crates register commands:

```rust
app.run(move |cx| {
    command::init(cx);

    clipboard::init(cx);
});
```

## Register One Command

```rust
use command::{Command, CommandRegistry};
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

## Register Multiple Commands

```rust
use command::{Command, CommandRegistry};
use gpui::App;

pub fn init(cx: &mut App) {
    CommandRegistry::global(cx).update(cx, |registry, _cx| {
        registry.register_commands([
            Command::new("example.open", "Open Example"),
            Command::new("example.close", "Close Example"),
        ]);
    });
}
```

## Register A Provider

Use `CommandProvider` when a feature crate owns a named command source.

```rust
use command::{Command, CommandProvider, CommandRegistry};
use gpui::App;

pub fn init(cx: &mut App) {
    CommandRegistry::global(cx).update(cx, |registry, cx| {
        registry.register_provider(ExampleProvider, cx);
    });
}

pub struct ExampleProvider;

impl CommandProvider for ExampleProvider {
    type Commands = Vec<Command>;

    fn commands(&self, _cx: &mut gpui::Context<CommandRegistry>) -> Self::Commands {
        vec![
            Command::new("example.open", "Open Example"),
            Command::new("example.close", "Close Example"),
        ]
    }
}
```

`CommandProvider::Commands` can be any `IntoIterator<Item = Command>`, such as `Vec<Command>`, an array, or a custom iterator.

## Read Registered Commands

```rust
use command::CommandRegistry;
use gpui::App;

pub fn inspect(cx: &mut App) {
    CommandRegistry::global(cx).read(cx).iter().for_each(|command| {
        println!("{}", command.id);
    });
}
```

Duplicate command ids are skipped and logged as warnings.
