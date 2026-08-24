use windows::Win32::Foundation::HWND;

#[derive(Clone, Copy, Default)]
pub(super) struct Controls {
    pub(super) title_label: HWND,
    pub(super) status: HWND,
    pub(super) status_detail: HWND,
    pub(super) issue_details_button: HWND,
    pub(super) target_list: HWND,
    pub(super) target_empty_title: HWND,
    pub(super) target_empty_hint: HWND,
    pub(super) process_source_button: HWND,
    pub(super) toggle_process_details_button: HWND,
    pub(super) pid_details_help_button: HWND,
    pub(super) add_selected_button: HWND,
    pub(super) settings_button: HWND,
}

impl Controls {
    pub(super) fn all(self) -> [HWND; 12] {
        [
            self.title_label,
            self.status,
            self.status_detail,
            self.issue_details_button,
            self.target_list,
            self.target_empty_title,
            self.target_empty_hint,
            self.process_source_button,
            self.toggle_process_details_button,
            self.pid_details_help_button,
            self.add_selected_button,
            self.settings_button,
        ]
    }
}
