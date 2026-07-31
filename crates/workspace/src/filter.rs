use module::Command;
use nucleo::{
    Config, Matcher, Utf32Str,
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
};

/// A command together with its zlaunch-compatible fuzzy score.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilteredCommand {
    pub index: usize,
    pub score: u16,
}

pub struct CommandFilter {
    matcher: Matcher,
}

impl Default for CommandFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandFilter {
    pub fn new() -> Self {
        let matcher = Matcher::new(Config::DEFAULT.match_paths());
        Self { matcher }
    }

    pub fn filter_with_scores<'a>(
        &mut self,
        commands: impl IntoIterator<Item = &'a Command>,
        query: &str,
    ) -> Vec<FilteredCommand> {
        let items = commands.into_iter();
        if query.trim().is_empty() {
            return items
                .enumerate()
                .map(|(index, _)| FilteredCommand { index, score: 0 })
                .collect();
        }

        let pattern = Atom::new(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut buf = Vec::new();
        let mut scored: Vec<FilteredCommand> = items
            .enumerate()
            .filter_map(|(index, command)| {
                self.score_command(&pattern, command, &mut buf)
                    .map(|score| FilteredCommand { index, score })
            })
            .collect();

        scored.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.index.cmp(&right.index))
        });

        scored
    }

    fn score_command(
        &mut self,
        pattern: &Atom,
        command: &Command,
        buf: &mut Vec<char>,
    ) -> Option<u16> {
        let scores = command
            .search_terms()
            .into_iter()
            .map(|needle| pattern.score(Utf32Str::new(&needle, buf), &mut self.matcher));

        let highest = scores.flatten().max();

        highest
    }
}
