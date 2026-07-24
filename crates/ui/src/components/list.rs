use gpui::{App, Div, SharedString, div, prelude::*};
use gpui_component::{ActiveTheme, IndexPath, list::ListItem};

/// Presentation contract for an item shown in a standard Keyway list.
///
/// Feature crates own their item data and only provide the text displayed by
/// the shared list component. Filtering and actions remain feature-specific.
pub trait ItemList {
    fn title(&self) -> SharedString;

    fn subtitle(&self) -> Option<SharedString> {
        None
    }

    fn icon(&self) -> Option<SharedString> {
        None
    }
}

/// A named group of indices into an [`ItemListState`]'s items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemListSection {
    title: Option<SharedString>,
    item_indices: Vec<usize>,
}

impl ItemListSection {
    pub fn new(
        title: impl Into<SharedString>,
        item_indices: impl IntoIterator<Item = usize>,
    ) -> Self {
        Self {
            title: Some(title.into()),
            item_indices: item_indices.into_iter().collect(),
        }
    }

    pub fn untitled(item_indices: impl IntoIterator<Item = usize>) -> Self {
        Self {
            title: None,
            item_indices: item_indices.into_iter().collect(),
        }
    }

    pub fn title(&self) -> Option<&SharedString> {
        self.title.as_ref()
    }

    pub fn item_indices(&self) -> &[usize] {
        &self.item_indices
    }
}

/// Reusable item, section, selection, and rendering state for Keyway lists.
pub struct ItemListState<T: ItemList> {
    items: Vec<T>,
    sections: Vec<ItemListSection>,
    selected: Option<IndexPath>,
}

impl<T: ItemList> ItemListState<T> {
    pub fn new(items: Vec<T>) -> Self {
        let sections = vec![ItemListSection::untitled(0..items.len())];
        Self {
            items,
            sections,
            selected: None,
        }
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn replace_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.sections = vec![ItemListSection::untitled(0..self.items.len())];
        self.selected = None;
    }

    pub fn set_sections(&mut self, sections: impl IntoIterator<Item = ItemListSection>) {
        let item_count = self.items.len();
        self.sections = sections
            .into_iter()
            .map(|mut section| {
                section.item_indices.retain(|index| *index < item_count);
                section
            })
            .collect();
        self.selected = None;
    }

    pub fn sections_count(&self) -> usize {
        self.sections.len().max(1)
    }

    pub fn items_count(&self, section: usize) -> usize {
        self.sections
            .get(section)
            .map(|section| section.item_indices.len())
            .unwrap_or(0)
    }

    pub fn item_at(&self, index: IndexPath) -> Option<&T> {
        let item_index = *self
            .sections
            .get(index.section)?
            .item_indices
            .get(index.row)?;
        self.items.get(item_index)
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.selected.and_then(|index| self.item_at(index))
    }

    pub fn set_selected_index(&mut self, selected: Option<IndexPath>) {
        self.selected = selected.filter(|index| self.item_at(*index).is_some());
    }

    pub fn render_item(&self, index: IndexPath, cx: &App) -> Option<ListItem> {
        let item = self.item_at(index)?;
        let selected = self.selected == Some(index);

        let mut row = div().flex().items_center().gap_2().min_w_0();
        if let Some(icon) = item.icon() {
            row = row.child(
                div()
                    .size_8()
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_sm()
                    .bg(cx.theme().secondary)
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(icon),
            );
        }

        let mut content = div()
            .min_w_0()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(item.title()),
            );
        if let Some(subtitle) = item.subtitle() {
            content = content.child(
                div()
                    .w_full()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(subtitle),
            );
        }

        row = row.child(content);

        Some(
            ListItem::new((
                "keyway-item-list-row",
                row_element_id(index.section, index.row),
            ))
            .selected(selected)
            .child(row),
        )
    }

    pub fn render_section_header(&self, section: usize, cx: &App) -> Option<Div> {
        let title = self.sections.get(section)?.title.clone()?;
        Some(
            div()
                .px_3()
                .py_1()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(title),
        )
    }
}

fn row_element_id(section: usize, row: usize) -> u64 {
    ((section as u64) << 32) | row as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestItem(&'static str);

    impl ItemList for TestItem {
        fn title(&self) -> SharedString {
            self.0.into()
        }
    }

    #[test]
    fn maps_section_rows_to_items_and_selection() {
        let mut state = ItemListState::new(vec![TestItem("first"), TestItem("second")]);
        state.set_sections([
            ItemListSection::new("Recommended", [1]),
            ItemListSection::new("All", [0, 1]),
        ]);

        let selected = IndexPath::new(0).section(0);
        state.set_selected_index(Some(selected));

        assert_eq!(state.sections_count(), 2);
        assert_eq!(state.items_count(1), 2);
        assert_eq!(state.selected_item().map(|item| item.0), Some("second"));
    }

    #[test]
    fn ignores_out_of_bounds_section_indices() {
        let mut state = ItemListState::new(vec![TestItem("only")]);
        state.set_sections([ItemListSection::untitled([0, 4])]);

        assert_eq!(state.items_count(0), 1);
        state.set_selected_index(Some(IndexPath::new(1)));
        assert!(state.selected_item().is_none());
    }
}
