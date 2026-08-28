use super::*;

pub(super) unsafe fn execute_main_window_action(
    state: &RefCell<AppWindow>,
    action: MainWindowAction,
) {
    match action {
        MainWindowAction::OpenSettings(request) => {
            let result = unsafe {
                prompt_settings(
                    request.parent,
                    request.instance,
                    request.icons,
                    request.initial,
                    |update| {
                        let mut app = state.borrow_mut();
                        match update {
                            SettingsLiveUpdate::Language(language) => {
                                app.apply_settings_language(language)
                            }
                            SettingsLiveUpdate::Theme(theme) => app.apply_settings_theme(theme),
                        }
                    },
                )
            };
            let warning =
                {
                    let mut app = state.borrow_mut();
                    app.finish_settings_window_modal();
                    match result {
                        Ok(Some(changes)) => {
                            app.apply_settings_changes(changes);
                            None
                        }
                        Ok(None) => None,
                        Err(error) => Some(app.action_failed_request(
                            "settings-window-failed",
                            Some(error.to_string()),
                        )),
                    }
                };
            if let Some(warning) = warning {
                unsafe {
                    let _ = show_message_dialog(warning);
                }
            }
        }
        MainWindowAction::EditTargetNote(request) => {
            let result = unsafe {
                prompt_target_note(
                    request.parent,
                    request.instance,
                    request.icon,
                    request.language,
                    request.theme,
                    &request.display_name,
                    request.current_note.as_deref(),
                )
            };
            let warning =
                match result {
                    Ok(Some(note)) => {
                        state.borrow_mut().apply_target_note_dialog_result(
                            &request.target_name,
                            request.target_pid,
                            note,
                        );
                        None
                    }
                    Ok(None) => None,
                    Err(error) => Some(state.borrow().action_failed_request(
                        "target-note-window-failed",
                        Some(error.to_string()),
                    )),
                };
            if let Some(warning) = warning {
                unsafe {
                    let _ = show_message_dialog(warning);
                }
            }
        }
        MainWindowAction::ShowMessage(request) => unsafe {
            let _ = show_message_dialog(request);
        },
        MainWindowAction::ShowIssues(issues) => {
            for issue in issues {
                let request = {
                    let app = state.borrow();
                    if !app.issues.visible_issues().any(|visible| visible == issue) {
                        continue;
                    }
                    app.issue_message_request(issue, app.issue_diagnostics.detail(issue))
                };
                let ignore_issue = request.ignore_issue;
                match unsafe { show_message_dialog(request) } {
                    IssueDialogResult::Dismissed => break,
                    IssueDialogResult::Acknowledged => {
                        if issue.clears_on_acknowledge() {
                            state.borrow_mut().clear_issue(issue);
                        }
                    }
                    IssueDialogResult::Ignored => {
                        if let Some(issue) = ignore_issue
                            && let Some(notice) = state.borrow_mut().ignore_issue(issue)
                        {
                            unsafe {
                                let _ = show_info_dialog(
                                    notice.parent,
                                    notice.icons,
                                    notice.language,
                                    notice.theme,
                                    &notice.title,
                                    &notice.body,
                                );
                            }
                        }
                    }
                }
            }
        }
        MainWindowAction::ShowInfo(request) => unsafe {
            let _ = show_info_dialog(
                request.parent,
                request.icons,
                request.language,
                request.theme,
                &request.title,
                &request.body,
            );
        },
        MainWindowAction::ShowTrayMenu(request) => {
            if let Some(command) = unsafe { show_tray_menu(request) } {
                let next_action = state.borrow_mut().command(command, 0, HWND::default());
                if let Some(next_action) = next_action {
                    unsafe {
                        execute_main_window_action(state, next_action);
                    }
                }
            }
        }
        MainWindowAction::ShowTargetContextMenu(request) => {
            if let Some(command) = unsafe { show_target_context_menu(&request) } {
                let next_action = {
                    let mut app = state.borrow_mut();
                    match command {
                        ID_TARGET_CONTEXT_TOGGLE_ENABLED => {
                            app.toggle_target_enabled_by_identity(
                                &request.target_name,
                                request.target_pid,
                            );
                            None
                        }
                        ID_TARGET_CONTEXT_EDIT_NOTE => app
                            .prepare_target_note_dialog_by_identity(
                                &request.target_name,
                                request.target_pid,
                            )
                            .map(MainWindowAction::EditTargetNote),
                        ID_REMOVE => {
                            app.remove_target_by_identity(&request.target_name, request.target_pid);
                            None
                        }
                        _ => None,
                    }
                };
                if let Some(next_action) = next_action {
                    unsafe {
                        execute_main_window_action(state, next_action);
                    }
                }
            }
        }
        MainWindowAction::ShowMainWindow => {
            let hwnd = state.borrow().hwnd;
            show_main_window(hwnd);
        }
        MainWindowAction::HideToTray => {
            let hwnd = state.borrow().hwnd;
            unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
        MainWindowAction::Close => {
            let warning = state.borrow_mut().prepare_for_destroy();
            if let Some(warning) = warning {
                unsafe {
                    let _ = show_message_dialog(warning);
                }
            }
            let hwnd = state.borrow().hwnd;
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        }
        MainWindowAction::HandoffClose => {
            let hwnd = {
                let mut app = state.borrow_mut();
                if !app.finish_handoff() {
                    return;
                }
                app.hwnd
            };
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        }
    }
}

unsafe fn show_message_dialog(request: MessageDialogRequest) -> IssueDialogResult {
    match unsafe {
        show_issue_dialog(
            request.parent,
            request.icons,
            request.language,
            request.theme,
            IssueDialogContent {
                title: &request.title,
                summary: &request.summary,
                explanation: request.explanation.as_deref(),
                detail: request.detail.as_deref(),
                diagnostic_code: request.diagnostic_code,
                allow_ignore: request.ignore_issue.is_some(),
            },
        )
    } {
        Ok(result) => result,
        Err(_) => {
            let title = to_wide(&request.title);
            let body = to_wide(&issue_message_body(
                &request.summary,
                request.explanation.as_deref(),
                request.detail.as_deref(),
            ));
            unsafe {
                let _ = MessageBoxW(
                    Some(request.parent),
                    PCWSTR(body.as_ptr()),
                    PCWSTR(title.as_ptr()),
                    MB_OK | MB_ICONERROR,
                );
            }
            IssueDialogResult::Acknowledged
        }
    }
}

unsafe fn show_tray_menu(request: TrayMenuRequest) -> Option<i32> {
    let menu = unsafe { PopupMenu::create() }?;
    let mut text_buffer = Vec::new();

    for (text, flags, id) in [
        (&request.status, MF_STRING | MF_GRAYED, 0usize),
        (
            &request.visibility_label,
            MF_STRING,
            request.visibility_command as usize,
        ),
        (&request.pause_label, MF_STRING, ID_PAUSE as usize),
        (&request.quit_label, MF_STRING, ID_QUIT as usize),
    ] {
        write_wide_buffer(text, &mut text_buffer);
        unsafe {
            let _ = AppendMenuW(menu.handle(), flags, id, PCWSTR(text_buffer.as_ptr()));
        }
        if id == 0 || id == ID_PAUSE as usize {
            unsafe {
                let _ = AppendMenuW(menu.handle(), MF_SEPARATOR, 0, PCWSTR::null());
            }
        }
    }

    let mut point = POINT::default();
    if unsafe { GetCursorPos(&mut point) }.is_err() {
        return None;
    }
    unsafe {
        let _ = SetForegroundWindow(request.parent);
    }
    let flags = TRACK_POPUP_MENU_FLAGS(TPM_RIGHTBUTTON.0 | TPM_RETURNCMD.0 | TPM_NONOTIFY.0);
    let command = unsafe {
        TrackPopupMenu(
            menu.handle(),
            flags,
            point.x,
            point.y,
            None,
            request.parent,
            None,
        )
        .0
    };
    // Let the notification-area owner finish dismissing the popup before the
    // next tray interaction. This is the sequence recommended for tray menus.
    unsafe {
        let _ = PostMessageW(Some(request.parent), WM_NULL, WPARAM(0), LPARAM(0));
    }
    (command != 0).then_some(command as i32)
}

unsafe fn show_target_context_menu(request: &TargetContextMenuRequest) -> Option<i32> {
    let menu = unsafe { PopupMenu::create() }?;
    let mut text_buffer = Vec::new();
    for (text, id) in [
        (
            &request.toggle_label,
            ID_TARGET_CONTEXT_TOGGLE_ENABLED as usize,
        ),
        (&request.edit_label, ID_TARGET_CONTEXT_EDIT_NOTE as usize),
        (&request.remove_label, ID_REMOVE as usize),
    ] {
        write_wide_buffer(text, &mut text_buffer);
        unsafe {
            let _ = AppendMenuW(menu.handle(), MF_STRING, id, PCWSTR(text_buffer.as_ptr()));
        }
        if id != ID_REMOVE as usize {
            unsafe {
                let _ = AppendMenuW(menu.handle(), MF_SEPARATOR, 0, PCWSTR::null());
            }
        }
    }

    unsafe {
        let _ = SetForegroundWindow(request.parent);
    }
    let flags = TRACK_POPUP_MENU_FLAGS(TPM_RIGHTBUTTON.0 | TPM_RETURNCMD.0 | TPM_NONOTIFY.0);
    let selected = unsafe {
        TrackPopupMenu(
            menu.handle(),
            flags,
            request.x,
            request.y,
            None,
            request.parent,
            None,
        )
        .0
    };
    (selected != 0).then_some(selected as i32)
}
