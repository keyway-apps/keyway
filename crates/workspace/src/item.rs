use std::any::TypeId;

use gpui_kit::{AnyEntity, AnyView, App, Context, EntityId, EventEmitter, Focusable, Render, Subscription, Window};

use crate::Workspace;

pub trait Item: Focusable + EventEmitter<Self::Event> + Render + Sized {
    type Event;
}

pub trait ItemHandle: 'static + Send {
    fn boxed_clone(&self) -> Box<dyn ItemHandle>;
    fn act_as_type(&self, type_id: TypeId, cx: &App) -> Option<AnyEntity>;
    fn subscribe_events(&self, item_id: EntityId, cx: &mut Context<Workspace>) -> Subscription;
    fn focus(&self, window: &mut Window, cx: &mut App);
    fn item_id(&self) -> EntityId;
    fn to_any_view(&self) -> AnyView;
}
