use module::Command;
use nucleo::{
    Config, Matcher, Utf32Str,
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
};
use regex::Regex;

/// A command together with its zlaunch-compatible fuzzy score.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilteredCommand {
    pub index: usize,
    pub score: u16,
}

pub struct CommandFilter {
    matcher: Matcher,
    command_regexes: Vec<Vec<Regex>>,
}

impl CommandFilter {
    pub fn new(commands: &[Command]) -> Self {
        let mut filter = Self {
            matcher: Matcher::new(Config::DEFAULT.match_paths()),
            command_regexes: Vec::new(),
        };
        filter.replace_commands(commands);
        filter
    }

    pub fn replace_commands(&mut self, commands: &[Command]) {
        self.command_regexes = commands
            .iter()
            .map(|command| {
                command
                    .match_regexes
                    .iter()
                    .flatten()
                    .filter_map(|pattern| match Regex::new(pattern) {
                        Ok(regex) => Some(regex),
                        Err(error) => {
                            tracing::warn!(
                                command_id = %command.id,
                                pattern,
                                %error,
                                "ignoring invalid command match regular expression"
                            );
                            None
                        }
                    })
                    .collect()
            })
            .collect();
    }

    pub fn filter_with_scores<'a>(
        &mut self,
        commands: impl IntoIterator<Item = &'a Command>,
        query: &str,
    ) -> Vec<FilteredCommand> {
        let commands = commands.into_iter();
        let query = query.trim();
        if query.is_empty() {
            return commands
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
        let mut scored: Vec<FilteredCommand> = commands
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

    pub fn regex_matches(&self, query: &str) -> Vec<usize> {
        let query = query.trim();
        if query.is_empty() {
            return Vec::new();
        }

        self.command_regexes
            .iter()
            .enumerate()
            .filter_map(|(index, regexes)| {
                regexes
                    .iter()
                    .any(|regex| regex.is_match(query))
                    .then_some(index)
            })
            .collect()
    }

    fn score_command(
        &mut self,
        pattern: &Atom,
        command: &Command,
        buf: &mut Vec<char>,
    ) -> Option<u16> {
        command
            .search_terms()
            .into_iter()
            .filter_map(|needle| pattern.score(Utf32Str::new(needle, buf), &mut self.matcher))
            .max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_regexes_are_compiled_once_and_match_in_command_order() {
        let commands = vec![
            Command::builder("url", "Open URL")
                .match_regex(r"^https?://")
                .build(),
            Command::builder("email", "Send Email")
                .match_regex(r"^[^@]+@[^@]+$")
                .build(),
        ];
        let filter = CommandFilter::new(&commands);

        assert_eq!(filter.regex_matches("https://keyway.app"), [0]);
        assert_eq!(filter.regex_matches("dev@keyway.app"), [1]);
        assert!(filter.regex_matches("plain text").is_empty());
    }

    #[test]
    fn invalid_regexes_do_not_hide_valid_ones() {
        let commands = vec![
            Command::builder("url", "Open URL")
                .match_regex("(")
                .match_regex(r"^https?://")
                .build(),
        ];
        let filter = CommandFilter::new(&commands);

        assert_eq!(filter.regex_matches("https://keyway.app"), [0]);
    }
}
