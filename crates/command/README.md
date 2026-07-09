## 注册 CommandProvider

```rust
use command::{command, Command, CommandProvider};

pub struct ExampleProvider;

impl CommandProvider for ExampleProvider {
    fn commands(&self) -> &[Command] {
        &[]
    }
}

command!(ExampleProvider);
```
