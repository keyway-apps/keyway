use module::AnyDynamic;

#[derive(Clone)]
pub struct DynamicItems {
    items: Vec<AnyDynamic>,
}

impl DynamicItems {
    pub fn new(items: Vec<AnyDynamic>) -> Self {
        Self { items }
    }

    pub fn process_query(&mut self, query: &str) {

    }
}