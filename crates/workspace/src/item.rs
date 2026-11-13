use std::any::TypeId;

use gpui_kit::{
    AnyEntity, AnyView, App, Context, Entity, EntityId, EventEmitter, FocusHandle, Focusable, Render, Subscription,
    Window,
};

use crate::Workspace;

pub trait Item: Focusable + EventEmitter<Self::Event> + Render + Sized {
    type Event;

    fn act_as_type<'a>(&'a self, type_id: TypeId, self_handle: &'a Entity<Self>, _: &'a App) -> Option<AnyEntity> {
        if TypeId::of::<Self>() == type_id {
            Some(self_handle.clone().into())
        } else {
            None
        }
    }
}

pub trait ItemHandle: 'static + Send {
    fn item_focus_handle(&self, cx: &App) -> FocusHandle;
    fn boxed_clone(&self) -> Box<dyn ItemHandle>;
    fn item_id(&self) -> EntityId;
    fn to_any_view(&self) -> AnyView;
    fn act_as_type(&self, type_id: TypeId, cx: &App) -> Option<AnyEntity>;
}

impl dyn ItemHandle {
    pub fn downcast<V: 'static>(&self) -> Option<Entity<V>> {
        self.to_any_view().downcast().ok()
    }

    pub fn act_as<V: 'static>(&self, cx: &App) -> Option<Entity<V>> {
        self.act_as_type(TypeId::of::<V>(), cx).and_then(|t| t.downcast().ok())
    }
}

impl<T: Item> ItemHandle for Entity<T> {
    fn item_focus_handle(&self, cx: &App) -> FocusHandle {
        self.read(cx).focus_handle(cx)
    }

    fn boxed_clone(&self) -> Box<dyn ItemHandle> {
        Box::new(self.clone())
    }

    fn item_id(&self) -> EntityId {
        self.entity_id()
    }

    fn to_any_view(&self) -> AnyView {
        self.clone().into()
    }

    fn act_as_type<'a>(&'a self, type_id: TypeId, cx: &'a App) -> Option<AnyEntity> {
        self.read(cx).act_as_type(type_id, self, cx)
    }
}

impl Clone for Box<dyn ItemHandle> {
    fn clone(&self) -> Box<dyn ItemHandle> {
        self.boxed_clone()
    }
}
