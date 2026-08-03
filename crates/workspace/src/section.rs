use crate::filter::FilteredCommand;

const SUGGESTIONS_LIMIT: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionType {
    Dynamic,
    Suggestions,
    BestMatch,
    All,
}

impl SectionType {
    pub fn title(self) -> Option<&'static str> {
        match self {
            Self::Dynamic => None,
            Self::Suggestions => Some("Suggestions"),
            Self::BestMatch => Some("Best Match"),
            Self::All => Some("All"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionItem {
    Dynamic,
    Command { index: usize },
}

#[derive(Clone, Debug, Default)]
pub struct SectionManager {
    sections: Vec<SectionType>,
    keyword_matches: Vec<FilteredCommand>,
    regex_matches: Vec<usize>,
    all_count: usize,
    query_is_empty: bool,
}

impl SectionManager {
    pub fn update(
        &mut self,
        query: &str,
        keyword_matches: Vec<FilteredCommand>,
        regex_matches: Vec<usize>,
        has_dynamic: bool,
        all_count: usize,
    ) {
        self.keyword_matches = keyword_matches;
        self.regex_matches = regex_matches;
        self.all_count = all_count;
        self.query_is_empty = query.trim().is_empty();
        self.sections.clear();

        if self.query_is_empty {
            if self.all_count > 0 {
                self.sections.push(SectionType::Suggestions);
                self.sections.push(SectionType::All);
            }
            return;
        }

        if !self.keyword_matches.is_empty() {
            self.sections.push(SectionType::BestMatch);
        } else {
            if has_dynamic {
                self.sections.push(SectionType::Dynamic);
            }
            if !self.regex_matches.is_empty() {
                self.sections.push(SectionType::BestMatch);
            }
        }

        if self.all_count > 0 {
            self.sections.push(SectionType::All);
        }
    }

    pub fn sections_count(&self) -> usize {
        self.sections.len().max(1)
    }

    pub fn section_type_at(&self, section: usize) -> Option<SectionType> {
        self.sections.get(section).copied()
    }

    pub fn items_count(&self, section: usize) -> usize {
        self.section_type_at(section)
            .map(|section| self.section_item_count(section))
            .unwrap_or(0)
    }

    pub fn item_at(&self, section: usize, row: usize) -> Option<SectionItem> {
        match self.section_type_at(section)? {
            SectionType::Dynamic => (row == 0).then_some(SectionItem::Dynamic),
            SectionType::Suggestions => self
                .keyword_matches
                .get(row)
                .map(|item| SectionItem::Command { index: item.index }),
            SectionType::BestMatch if !self.keyword_matches.is_empty() => self
                .keyword_matches
                .get(row)
                .map(|item| SectionItem::Command { index: item.index }),
            SectionType::BestMatch => self
                .regex_matches
                .get(row)
                .copied()
                .map(|index| SectionItem::Command { index }),
            SectionType::All => {
                (row < self.all_count).then_some(SectionItem::Command { index: row })
            }
        }
    }

    pub fn first_item(&self) -> Option<(usize, usize)> {
        self.sections
            .iter()
            .enumerate()
            .find(|(_, section)| self.section_item_count(**section) > 0)
            .map(|(section, _)| (section, 0))
    }

    fn section_item_count(&self, section: SectionType) -> usize {
        match section {
            SectionType::Dynamic => 1,
            SectionType::Suggestions => self.keyword_matches.len().min(SUGGESTIONS_LIMIT),
            SectionType::BestMatch if !self.keyword_matches.is_empty() => {
                self.keyword_matches.len()
            }
            SectionType::BestMatch => self.regex_matches.len(),
            SectionType::All => self.all_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(index: usize, score: u16) -> FilteredCommand {
        FilteredCommand { index, score }
    }

    #[test]
    fn empty_query_has_suggestions_and_all() {
        let mut manager = SectionManager::default();
        manager.update(
            "",
            (0..6).map(|index| result(index, 0)).collect(),
            Vec::new(),
            false,
            6,
        );

        assert_eq!(
            manager.sections,
            [SectionType::Suggestions, SectionType::All]
        );
        assert_eq!(manager.items_count(0), 5);
        assert_eq!(
            manager.item_at(0, 4),
            Some(SectionItem::Command { index: 4 })
        );
    }

    #[test]
    fn keyword_matches_take_priority_over_dynamic_and_regex_matches() {
        let mut manager = SectionManager::default();
        manager.update("term", vec![result(2, 10)], vec![1], true, 3);

        assert_eq!(manager.sections, [SectionType::BestMatch, SectionType::All]);
        assert_eq!(
            manager.item_at(0, 0),
            Some(SectionItem::Command { index: 2 })
        );
    }

    #[test]
    fn dynamic_precedes_regex_best_matches() {
        let mut manager = SectionManager::default();
        manager.update("https://keyway.app", Vec::new(), vec![1], true, 3);

        assert_eq!(
            manager.sections,
            [
                SectionType::Dynamic,
                SectionType::BestMatch,
                SectionType::All
            ]
        );
        assert_eq!(manager.item_at(0, 0), Some(SectionItem::Dynamic));
        assert_eq!(
            manager.item_at(1, 0),
            Some(SectionItem::Command { index: 1 })
        );
    }

    #[test]
    fn regex_matches_remain_best_matches_without_dynamic() {
        let mut manager = SectionManager::default();
        manager.update("https://keyway.app", Vec::new(), vec![0], false, 1);

        assert_eq!(manager.sections, [SectionType::BestMatch, SectionType::All]);
        assert_eq!(manager.first_item(), Some((0, 0)));
    }
}
