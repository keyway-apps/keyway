use super::command_filter::FilteredCommand;

/// Sections displayed by the command palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionType {
    BestMatch,
    All,
    Recommended,
}

impl SectionType {
    pub fn title(self) -> &'static str {
        match self {
            Self::BestMatch => "Best Match",
            Self::All => "All",
            Self::Recommended => "Recommended",
        }
    }
}

/// Keeps section membership and converts section rows to filtered result rows.
#[derive(Clone, Debug, Default)]
pub struct SectionManager {
    filtered: Vec<FilteredCommand>,
    all_count: usize,
    query_is_empty: bool,
    recommended_count: usize,
}

impl SectionManager {
    pub fn update(&mut self, filtered: Vec<FilteredCommand>, all_count: usize, query: &str) {
        self.filtered = filtered;
        self.all_count = all_count;
        self.query_is_empty = query.trim().is_empty();
        self.recommended_count = self
            .filtered
            .len()
            .min(if self.query_is_empty { 5 } else { 3 });
    }

    pub fn sections(&self) -> Vec<SectionType> {
        if self.all_count == 0 {
            return Vec::new();
        }
        if self.query_is_empty {
            vec![SectionType::Recommended, SectionType::All]
        } else {
            vec![SectionType::BestMatch, SectionType::All]
        }
    }

    pub fn section_item_count(&self, section: SectionType) -> usize {
        match section {
            SectionType::BestMatch => usize::from(!self.query_is_empty) * self.filtered.len(),
            SectionType::All => self.all_count,
            SectionType::Recommended => usize::from(self.query_is_empty) * self.recommended_count,
        }
    }

    pub fn filtered_index(&self, section: SectionType, row: usize) -> Option<usize> {
        match section {
            SectionType::BestMatch if !self.query_is_empty => {
                self.filtered.get(row).map(|result| result.index)
            }
            SectionType::All if row < self.all_count => Some(row),
            SectionType::Recommended if self.query_is_empty => {
                self.filtered.get(row).map(|result| result.index)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(index: usize) -> FilteredCommand {
        FilteredCommand { index, score: 1 }
    }

    #[test]
    fn query_sections_include_best_match() {
        let mut manager = SectionManager::default();
        manager.update(vec![result(2), result(1)], 3, "term");
        assert_eq!(
            manager.sections(),
            [SectionType::BestMatch, SectionType::All]
        );
        assert_eq!(manager.section_item_count(SectionType::BestMatch), 2);
        assert_eq!(manager.filtered_index(SectionType::BestMatch, 0), Some(2));
        assert_eq!(manager.section_item_count(SectionType::All), 3);
        assert_eq!(manager.filtered_index(SectionType::All, 2), Some(2));
    }

    #[test]
    fn empty_query_has_all_and_recommended() {
        let mut manager = SectionManager::default();
        manager.update((0..6).map(result).collect(), 6, "");
        assert_eq!(
            manager.sections(),
            [SectionType::Recommended, SectionType::All]
        );
        assert_eq!(manager.section_item_count(SectionType::Recommended), 5);
    }
}
