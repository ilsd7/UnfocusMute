use windows::Win32::Foundation::HWND;

#[derive(Clone, Copy, Default)]
pub(super) struct Controls {
    pub(super) title_label: HWND,
    pub(super) subtitle_label: HWND,
    pub(super) status: HWND,
    pub(super) status_detail: HWND,
    pub(super) targets_label: HWND,
    pub(super) target_list: HWND,
    pub(super) add_label: HWND,
    pub(super) running_label: HWND,
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
    pub(super) fn all(self) -> [HWND; 21] {
        [
            self.title_label,
            self.subtitle_label,
            self.status,
            self.status_detail,
            self.targets_label,
            self.target_list,
            self.add_label,
            self.running_label,
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
