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
                Some(display_pid_text(&name, pid)),
                Some(format!("{} {pid} pid", search_name.as_ref())),
            ),
            None if count > 1 => (
                Some(display_pid_count_text(&name, count)),
                Some(format!("{} {count} pid", search_name.as_ref())),
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
pub(super) struct SearchTerms<'a>(Vec<Cow<'a, str>>);

impl SearchTerms<'_> {
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn all(&self, mut predicate: impl FnMut(&str) -> bool) -> bool {
        self.0.iter().all(|term| predicate(term.as_ref()))
    }
}

fn lowercase_if_needed(text: &str) -> Cow<'_, str> {
    if text.is_ascii() {
        if text.bytes().any(|byte| byte.is_ascii_uppercase()) {
            Cow::Owned(text.to_ascii_lowercase())
        } else {
            Cow::Borrowed(text)
        }
    } else {
        let lowercase = text.to_lowercase();
        if lowercase == text {
            Cow::Borrowed(text)
        } else {
            Cow::Owned(lowercase)
        }
    }
}

fn into_owned_if_allocated(text: Cow<'_, str>) -> Option<String> {
    match text {
        Cow::Borrowed(_) => None,
        Cow::Owned(text) => Some(text),
    }
}

fn display_pid_text(text: &str, pid: u32) -> String {
    format!("{text} (PID {pid})")
}

fn display_pid_count_text(text: &str, count: usize) -> String {
    format!("{text} ({count} PID)")
}

pub(super) fn search_terms(query: &str) -> SearchTerms<'_> {
    SearchTerms(query.split_whitespace().map(lowercase_if_needed).collect())
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
    fn pid_choices_display_pid_before_number() {
        let choice = ProcessChoice::new("musicapp.exe".to_owned(), Some(4242), 1);

        assert_eq!(choice.display_name(), "musicapp.exe (PID 4242)");
    }

    #[test]
    fn grouped_choices_match_by_name_and_count() {
        let choice = ProcessChoice::new("Chat.exe".to_owned(), None, 3);

        assert!(choice.matches_search(&search_terms("chat 3")));
        assert_eq!(choice.display_name(), "Chat.exe (3 PID)");
    }

    #[test]
    fn lowercase_singleton_choices_need_no_allocated_search_text() {
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
    fn non_ascii_uppercase_search_terms_match_normalized_choices() {
        let choice = ProcessChoice::new("äpp.exe".to_owned(), None, 1);

        assert!(choice.matches_search(&search_terms("ÄPP")));
    }

    #[test]
    fn non_ascii_uppercase_choices_keep_lowercase_search_text() {
        let choice = ProcessChoice::new("ÄPP.EXE".to_owned(), None, 1);

        assert_eq!(choice.search_text.as_deref(), Some("äpp.exe"));
        assert!(choice.matches_search(&search_terms("ÄPP")));
    }

    #[test]
    fn search_terms_borrow_lowercase_input_without_allocating() {
        assert_eq!(search_terms("player").0, [Cow::Borrowed("player")]);
    }

    #[test]
    fn search_terms_collect_each_word() {
        assert_eq!(
            search_terms("player 4242").0,
            [Cow::Borrowed("player"), Cow::Borrowed("4242")]
        );
    }
}
