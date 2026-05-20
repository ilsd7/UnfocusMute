use std::borrow::Cow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ProcessChoice {
    pub(super) name: String,
    pub(super) pid: Option<u32>,
    display_name: Option<String>,
    search_text: Option<String>,
}

impl ProcessChoice {
    pub(super) fn new(name: String, pid: Option<u32>, count: usize) -> Self {
        let search_name = lowercase_if_needed(&name);
        let (display_name, search_text) = match pid {
            Some(pid) => (
                Some(format!("{name} (PID {pid})")),
                Some(format!("{search_name} pid {pid}")),
            ),
            None if count > 1 => (
                Some(format!("{name} ({count} PID)")),
                Some(format!("{search_name} {count} pid")),
            ),
            None => (None, into_owned_if_allocated(search_name)),
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

    pub(super) fn matches_search(&self, terms: &SearchTerms<'_>) -> bool {
        let search_text = self.search_text.as_deref().unwrap_or(&self.name);
        terms.all(|term| search_text.contains(term))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SearchTerms<'a> {
    Empty,
    One(Cow<'a, str>),
    Many(Vec<Cow<'a, str>>),
}

impl SearchTerms<'_> {
    pub(super) fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    fn all(&self, mut predicate: impl FnMut(&str) -> bool) -> bool {
        match self {
            Self::Empty => true,
            Self::One(term) => predicate(term.as_ref()),
            Self::Many(terms) => terms.iter().all(|term| predicate(term.as_ref())),
        }
    }
}

fn lowercase_if_needed(text: &str) -> Cow<'_, str> {
    if text.chars().any(char::is_uppercase) {
        Cow::Owned(text.to_lowercase())
    } else {
        Cow::Borrowed(text)
    }
}

fn into_owned_if_allocated(text: Cow<'_, str>) -> Option<String> {
    match text {
        Cow::Borrowed(_) => None,
        Cow::Owned(text) => Some(text),
    }
}

pub(super) fn search_terms(query: &str) -> SearchTerms<'_> {
    let mut terms = query.split_whitespace().map(lowercase_if_needed);
    let Some(first) = terms.next() else {
        return SearchTerms::Empty;
    };
    let Some(second) = terms.next() else {
        return SearchTerms::One(first);
    };

    let mut many = Vec::with_capacity(terms.size_hint().0 + 2);
    many.push(first);
    many.push(second);
    many.extend(terms);
    SearchTerms::Many(many)
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

    #[test]
    fn singleton_choices_reuse_name_for_search() {
        let choice = ProcessChoice::new("player.exe".to_owned(), None, 1);

        assert_eq!(choice.search_text, None);
        assert!(choice.matches_search(&search_terms("player")));
    }

    #[test]
    fn uppercase_singleton_choices_keep_lowercase_search_text() {
        let choice = ProcessChoice::new("Player.EXE".to_owned(), None, 1);

        assert_eq!(choice.search_text.as_deref(), Some("player.exe"));
        assert!(choice.matches_search(&search_terms("player")));
    }

    #[test]
    fn single_search_term_avoids_term_vec() {
        assert!(matches!(
            search_terms("player"),
            SearchTerms::One(Cow::Borrowed("player"))
        ));
    }
}
