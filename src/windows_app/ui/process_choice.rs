use std::borrow::Cow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ProcessChoice {
    pub(super) name: String,
    pub(super) pid: Option<u32>,
    display_name: Option<String>,
    search_text: String,
}

impl ProcessChoice {
    pub(super) fn new(name: String, pid: Option<u32>, count: usize) -> Self {
        let search_name = lowercase_if_needed(&name);
        let (display_name, search_text) = match pid {
            Some(pid) => (
                Some(format!("{name} (PID {pid})")),
                format!("{search_name} pid {pid}"),
            ),
            None if count > 1 => (
                Some(format!("{name} ({count} PID)")),
                format!("{search_name} {count} pid"),
            ),
            None => (None, search_name.into_owned()),
        };

        Self {
            name,
            pid,
            display_name,
            search_text,
        }
    }

    pub(super) fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }

    pub(super) fn matches_search(&self, terms: &[String]) -> bool {
        terms.iter().all(|term| self.search_text.contains(term))
    }
}

fn lowercase_if_needed(text: &str) -> Cow<'_, str> {
    if text.chars().any(char::is_uppercase) {
        Cow::Owned(text.to_lowercase())
    } else {
        Cow::Borrowed(text)
    }
}

pub(super) fn search_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|term| !term.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_choices_match_by_name_and_pid() {
        let choice = ProcessChoice::new("MusicApp.exe".to_owned(), Some(4242), 1);

        assert!(choice.matches_search(&search_terms("musicapp pid 4242")));
    }

    #[test]
    fn grouped_choices_match_by_name_and_count() {
        let choice = ProcessChoice::new("Chat.exe".to_owned(), None, 3);

        assert!(choice.matches_search(&search_terms("chat 3")));
    }
}
