use windows::Win32::Foundation::HWND;

#[derive(Clone, Copy, Default)]
pub(super) struct Controls {
    pub(super) title_label: HWND,
    pub(super) subtitle_label: HWND,
    pub(super) status: HWND,
    pub(super) status_detail: HWND,
    pub(super) target_list: HWND,
    pub(super) target_empty_title: HWND,
    pub(super) target_empty_hint: HWND,
    pub(super) running_hint: HWND,
    pub(super) running_combo: HWND,
    pub(super) refresh_button: HWND,
    pub(super) toggle_process_details_button: HWND,
    pub(super) pid_details_help_button: HWND,
    pub(super) add_selected_button: HWND,
    pub(super) manual_label: HWND,
    pub(super) manual_edit: HWND,
    pub(super) add_manual_button: HWND,
    pub(super) settings_button: HWND,
    pub(super) pause_button: HWND,
    pub(super) hide_button: HWND,
    pub(super) quit_button: HWND,
}

impl Controls {
    pub(super) fn all(self) -> [HWND; 20] {
        [
            self.title_label,
            self.subtitle_label,
            self.status,
            self.status_detail,
            self.target_list,
            self.target_empty_title,
            self.target_empty_hint,
            self.running_hint,
            self.running_combo,
            self.refresh_button,
            self.toggle_process_details_button,
            self.pid_details_help_button,
            self.add_selected_button,
            self.manual_label,
            self.manual_edit,
            self.add_manual_button,
            self.settings_button,
            self.pause_button,
            self.hide_button,
            self.quit_button,
        ]
    }
}
