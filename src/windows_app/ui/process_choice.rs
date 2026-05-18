#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ProcessChoice {
    pub(super) name: String,
    pub(super) pid: Option<u32>,
    display_name: String,
    search_text: String,
}

impl ProcessChoice {
    pub(super) fn new(name: String, pid: Option<u32>, count: usize) -> Self {
        let display_name = match pid {
            Some(pid) => format!("{name} (PID {pid})"),
            None if count > 1 => format!("{name} ({count} PID)"),
            None => name.clone(),
        };
        let search_text = match pid {
            Some(pid) => format!("{name} {display_name} {pid}").to_lowercase(),
            None => format!("{name} {display_name}").to_lowercase(),
        };

        Self {
            name,
            pid,
            display_name,
            search_text,
        }
    }

    pub(super) fn display_name(&self) -> &str {
        &self.display_name
    }

    pub(super) fn matches_search(&self, terms: &[String]) -> bool {
        terms.iter().all(|term| self.search_text.contains(term))
    }
}

pub(super) fn search_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|term| !term.is_empty())
        .collect()
}
