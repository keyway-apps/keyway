use module::Command;

use crate::filter::FilteredCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionType {
    Dynamic,
    Recommended,
    All,
    BestMatch,
}

#[derive(Clone, Debug, Default)]
pub struct SectionManager {
    has_dynamic: bool,
    all_count: usize,
    show_best_match: bool,
    best_match_count: Option<usize>,
}

impl SectionManager {
    pub fn new(show_best_match: bool) -> Self {
        Self {
            has_dynamic: false,
            all_count: 0,
            show_best_match,
            best_match_count: None,
        }
    }

    pub fn update(&mut self, items: &[Command]) {}

    pub fn update_with_scores(
        &mut self,
        items: &[Command],
        filtered: &[FilteredCommand],
        has_dynamic: bool,
    ) {
        self.has_dynamic = has_dynamic;

        self.all_count = items.len();
        self.best_match_count = None;

        if self.show_best_match && !filtered.is_empty() {
            self.compute_best_match(filtered);
        }
    }

    fn compute_best_match(&mut self, filtered: &[FilteredCommand]) {
        let count = filtered.iter().count();
        self.best_match_count = Some(count);
    }

    pub fn has_best_match(&self) -> bool {
        self.best_match_count.is_some()
    }

    pub fn sections_count(&self) -> usize {
        let mut count = 0;
        if self.has_best_match() {
            count += 1;
        }
        count
    }

    pub fn section_type_at(&self, section: usize) -> SectionType {
        let mut current_section = 0;

        if self.has_dynamic {
            if section == current_section {
                return SectionType::Dynamic;
            }
            current_section += 1;
        }

        SectionType::All
    }
}
