use gpui_kit::{AnyElement, SharedString, base::Selectable};

pub mod item;

pub trait Command: Selectable + Clone + 'static {
    fn id(&self) -> &SharedString;
    fn label(&self) -> &SharedString;

    fn content(&self) -> Option<AnyElement> {
        None
    }
}

pub trait CommandHandle {}
