use super::win32::storage_bytes_hint;
use std::borrow::Cow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ProcessChoice {
    pub(super) name: String,
    pub(super) pid: Option<u32>,
    display_name: Option<String>,
    display_storage_bytes: usize,
    search_text: Option<String>,
}

impl ProcessChoice {
    pub(super) fn new(name: String, pid: Option<u32>, count: usize) -> Self {
        let search_name = lowercase_if_needed(&name);
        let (display_name, search_text) = match pid {
            Some(pid) => (
                Some(display_pid_text(&name, pid)),
                Some(search_text_with_number(
                    search_name.as_ref(),
                    pid as usize,
                    "pid",
                )),
            ),
            None if count > 1 => (
                Some(display_pid_count_text(&name, count)),
                Some(search_text_with_number(search_name.as_ref(), count, "pid")),
            ),
            None => (None, into_owned_if_allocated(search_name)),
        };
        let display_storage_bytes = storage_bytes_hint(display_name.as_deref().unwrap_or(&name));

        Self {
            name,
            pid,
            display_name,
            display_storage_bytes,
            search_text,
        }
    }

    pub(super) fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }

    pub(super) fn display_storage_bytes(&self) -> usize {
        self.display_storage_bytes
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
    Two(Cow<'a, str>, Cow<'a, str>),
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
            Self::Two(first, second) => predicate(first.as_ref()) && predicate(second.as_ref()),
            Self::Many(terms) => terms.iter().all(|term| predicate(term.as_ref())),
        }
    }
}

fn lowercase_if_needed(text: &str) -> Cow<'_, str> {
    if text.bytes().any(|byte| byte.is_ascii_uppercase()) {
        Cow::Owned(text.to_ascii_lowercase())
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

fn display_pid_text(text: &str, pid: u32) -> String {
    let mut output = String::with_capacity(text.len() + 7 + decimal_digit_count(pid as usize));
    output.push_str(text);
    output.push_str(" (PID ");
    push_decimal(&mut output, pid as usize);
    output.push(')');
    output
}

fn display_pid_count_text(text: &str, count: usize) -> String {
    let mut output = String::with_capacity(text.len() + 7 + decimal_digit_count(count));
    output.push_str(text);
    output.push_str(" (");
    push_decimal(&mut output, count);
    output.push_str(" PID");
    output.push(')');
    output
}

fn search_text_with_number(text: &str, number: usize, label: &str) -> String {
    let mut output =
        String::with_capacity(text.len() + 2 + label.len() + decimal_digit_count(number));
    output.push_str(text);
    output.push(' ');
    push_decimal(&mut output, number);
    output.push(' ');
    output.push_str(label);
    output
}

fn push_decimal(output: &mut String, mut number: usize) {
    let mut digits = [0u8; 20];
    let mut len = 0;
    loop {
        digits[len] = b'0' + (number % 10) as u8;
        len += 1;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    for digit in digits[..len].iter().rev() {
        output.push(*digit as char);
    }
}

fn decimal_digit_count(number: usize) -> usize {
    if number == 0 {
        1
    } else {
        number.ilog10() as usize + 1
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
    let Some(third) = terms.next() else {
        return SearchTerms::Two(first, second);
    };

    let mut many = Vec::with_capacity(terms.size_hint().0 + 3);
    many.push(first);
    many.push(second);
    many.push(third);
    many.extend(terms);
    SearchTerms::Many(many)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn pid_choices_match_by_name_and_pid() {
        let choice = ProcessChoice::new("MusicApp.exe".to_owned(), Some(4242), 1);

        assert!(choice.matches_search(&search_terms("musicapp pid 4242")));
    }

    #[test]
    fn pid_choices_display_pid_before_number() {
        let choice = ProcessChoice::new("musicapp.exe".to_owned(), Some(4242), 1);

        assert_eq!(choice.display_name(), "musicapp.exe (PID 4242)");
        assert_eq!(
            choice.display_storage_bytes(),
            ("musicapp.exe (PID 4242)".len() + 1) * size_of::<u16>()
        );
    }

    #[test]
    fn grouped_choices_match_by_name_and_count() {
        let choice = ProcessChoice::new("Chat.exe".to_owned(), None, 3);

        assert!(choice.matches_search(&search_terms("chat 3")));
        assert_eq!(
            choice.display_storage_bytes(),
            ("Chat.exe (3 PID)".len() + 1) * size_of::<u16>()
        );
    }

    #[test]
    fn singleton_choices_reuse_name_for_search() {
        let choice = ProcessChoice::new("player.exe".to_owned(), None, 1);

        assert_eq!(choice.search_text, None);
        assert_eq!(
            choice.display_storage_bytes(),
            ("player.exe".len() + 1) * size_of::<u16>()
        );
        assert!(choice.matches_search(&search_terms("player")));
    }

    #[test]
    fn display_storage_bytes_counts_utf16_units() {
        let choice = ProcessChoice::new("게임.exe".to_owned(), None, 1);

        assert_eq!(
            choice.display_storage_bytes(),
            ("게임.exe".encode_utf16().count() + 1) * size_of::<u16>()
        );
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

    #[test]
    fn two_search_terms_avoid_term_vec() {
        assert!(matches!(
            search_terms("player 4242"),
            SearchTerms::Two(Cow::Borrowed("player"), Cow::Borrowed("4242"))
        ));
    }
}
