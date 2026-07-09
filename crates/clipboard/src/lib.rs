use command::prelude::*;

pub struct Clipboard;

impl CommandProvider for Clipboard {
    fn commands(&self) -> &[Command] {
        &[]
    }
}

command!(Clipboard);
