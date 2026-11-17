use std::rc::Rc;

use gpui_kit::{AnyElement, App, IntoElement, SharedString, Window, base::Selectable};
use serde::{Deserialize, Serialize};

use crate::Command;

#[derive(Clone, Serialize, Deserialize)]
pub struct CommandItem {
    #[serde(skip)]
    pub(crate) content: Option<Rc<CommandItemContent>>,
}

impl CommandItem {
    pub fn child<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&mut Window, &mut App) -> E + 'static,
        E: IntoElement,
    {
        self.content = Some(Rc::new(move |window, cx| builder(window, cx).into_any_element()));
        self
    }
}

impl Command for CommandItem {
    fn id(&self) -> &SharedString {
        todo!()
    }

    fn label(&self) -> &SharedString {
        todo!()
    }
}

impl Selectable for CommandItem {
    fn selected(mut self, selected: bool) -> Self {
        todo!()
    }

    fn is_selected(&self) -> bool {
        todo!()
    }
}

pub(crate) type CommandItemContent = dyn Fn(&mut Window, &mut App) -> AnyElement;
