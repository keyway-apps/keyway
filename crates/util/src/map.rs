use collections::HashMap;
use core::any::TypeId;

pub type TypeIdMap<V> = HashMap<TypeId, V>;
