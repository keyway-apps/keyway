use gpui::App;
use module::AnyDynamic;

#[derive(Clone)]
pub struct DynamicItems {
    items: Vec<AnyDynamic>,
    selected: Option<usize>,
}

impl DynamicItems {
    pub fn new(items: Vec<AnyDynamic>) -> Self {
        let mut this = Self {
            items,
            selected: None,
        };
        this.sort_items();
        this
    }

    pub fn replace(&mut self, items: Vec<AnyDynamic>) {
        self.items = items;
        self.selected = None;
        self.sort_items();
    }

    pub fn process_query(&mut self, query: &str, cx: &mut App) -> bool {
        self.selected = None;
        let mut highest_score = None;

        for (index, dynamic) in self.items.iter().enumerate() {
            let Some(score) = dynamic.score(query, cx) else {
                continue;
            };

            if highest_score.is_none_or(|best| score > best) {
                highest_score = Some(score);
                self.selected = Some(index);
            }
        }

        self.selected.is_some()
    }

    pub fn clear(&mut self) {
        self.selected = None;
    }

    pub fn selected(&self) -> Option<&AnyDynamic> {
        self.selected.and_then(|index| self.items.get(index))
    }

    fn sort_items(&mut self) {
        self.items.sort_by(|left, right| left.id().cmp(right.id()));
    }
}
