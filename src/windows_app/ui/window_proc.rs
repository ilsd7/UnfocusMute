use super::app_actions::execute_main_window_action;
use super::*;
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::UI::WindowsAndMessaging::WM_COPYDATA;

fn is_state_independent_flat_button(id: i32) -> bool {
    matches!(
        id,
        ID_PROCESS_SOURCE
            | ID_TOGGLE_PROCESS_DETAILS
            | ID_PID_DETAILS_HELP
            | ID_ISSUE_DETAILS
            | ID_ADD_SELECTED
    )
}

unsafe fn draw_state_independent_main_item(lparam: LPARAM) -> Option<LRESULT> {
    if lparam.0 == 0 {
        return None;
    }
    let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
    if !is_state_independent_flat_button(draw.CtlID as i32) {
        return None;
    }
    let font = unsafe { SendMessageW(draw.hwndItem, WM_GETFONT, None, None) }.0;
    if font == 0 {
        return None;
    }
    Some(LRESULT(
        unsafe { win32::draw_flat_button(draw, HGDIOBJ(font as *mut c_void)) } as isize,
    ))
}

unsafe fn state_independent_main_control_color(
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    let palette = active_palette();
    let background = match message {
        WM_CTLCOLOREDIT => palette.input,
        WM_CTLCOLORLISTBOX => palette.panel,
        WM_CTLCOLORBTN => {
            let control = HWND(lparam.0 as *mut c_void);
            if !control.0.is_null() && unsafe { GetDlgCtrlID(control) } == ID_PROCESS_SEARCH_TOGGLE
            {
                palette.input
            } else {
                palette.page
            }
        }
        _ => return None,
    };
    unsafe { win32::themed_control_color(wparam, palette.text, background) }
}

pub(super) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = lparam.0 as *const CREATESTRUCTW;
        if !create.is_null() {
            let app = unsafe { (*create).lpCreateParams as *mut RefCell<AppWindow> };
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, app as isize);
            }
        }
        return LRESULT(1);
    }
    if message == WM_REDRAW_DEFERRED_CONTROL {
        unsafe {
            win32::redraw_deferred_control(hwnd, wparam);
        }
        return LRESULT(0);
    }
    if message == WM_DRAWITEM
        && let Some(result) = unsafe { draw_state_independent_main_item(lparam) }
    {
        return result;
    }
    if let Some(result) = unsafe { state_independent_main_control_color(message, wparam, lparam) } {
        return result;
    }
    let state = unsafe {
        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
            as *mut RefCell<AppWindow>;
        ptr.as_ref()
    };

    if let Some(state) = state {
        let Ok(mut app) = state.try_borrow_mut() else {
            if let Some(result) =
                unsafe { win32::defer_reentrant_owner_draw(hwnd, message, lparam) }
            {
                return result;
            }
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        };
        if let Some(result) =
            default_button_message_result(message, wparam, &mut app.default_button_id)
        {
            return result;
        }
        if app.taskbar_created_message != 0 && message == app.taskbar_created_message {
            app.restore_tray_icon();
            return LRESULT(0);
        }
        if message == WM_COPYDATA {
            let Some(data) = (unsafe { (lparam.0 as *const COPYDATASTRUCT).as_ref() }) else {
                return LRESULT(0);
            };
            if data.dwData != handoff::COPYDATA_ID
                || data.cbData == 0
                || data.cbData as usize > handoff::MAX_BYTES
                || data.lpData.is_null()
            {
                return LRESULT(0);
            }
            let bytes = unsafe {
                std::slice::from_raw_parts(data.lpData.cast::<u8>(), data.cbData as usize)
            };
            let source = HWND(wparam.0 as *mut c_void);
            return LRESULT(isize::from(app.accept_handoff_snapshot(source, bytes)));
        }
        if message == WM_REQUEST_HANDOFF_STATE {
            let successor = HWND(wparam.0 as *mut c_void);
            if successor.0.is_null() || !single_instance::handoff_successor_is_valid(successor) {
                return LRESULT(0);
            }
            let Some(bytes) = app.prepare_handoff_snapshot() else {
                return LRESULT(0);
            };
            drop(app);
            let finish_queued =
                unsafe { PostMessageW(Some(hwnd), WM_FINISH_HANDOFF, WPARAM(0), LPARAM(0)) }
                    .is_ok();
            if !finish_queued {
                state.borrow_mut().abort_handoff();
                return LRESULT(0);
            }
            let transferred = single_instance::send_handoff_snapshot(hwnd, successor, &bytes);
            if !transferred {
                state.borrow_mut().abort_handoff();
            }
            return LRESULT(isize::from(transferred));
        }
        if message == WM_FINISH_HANDOFF {
            drop(app);
            unsafe {
                execute_main_window_action(state, MainWindowAction::HandoffClose);
            }
            return LRESULT(0);
        }
        if app.handoff.is_prepared()
            && matches!(
                message,
                WM_COMMAND | WM_CLOSE | WM_CONTEXTMENU | WM_TRAY_ICON
            )
        {
            return LRESULT(0);
        }
        match message {
            WM_CREATE => {
                if let Err(error) = unsafe { app.on_create(hwnd) } {
                    app.create_error = Some(error.to_string());
                    return LRESULT(-1);
                }
                return LRESULT(0);
            }
            WM_COMMAND => {
                let id = loword(wparam.0 as u32) as i32;
                let notification = hiword(wparam.0 as u32);
                let source = HWND(lparam.0 as *mut c_void);
                if id == ID_SHOW && notification == 0 {
                    drop(app);
                    show_main_window(hwnd);
                } else {
                    let action = app.command(id, notification, source);
                    drop(app);
                    if let Some(action) = action {
                        unsafe {
                            execute_main_window_action(state, action);
                        }
                    }
                }
                return LRESULT(0);
            }
            WM_TIMER => {
                app.timer_tick(wparam.0);
                return LRESULT(0);
            }
            WM_FOREGROUND_CHANGED => {
                FOREGROUND_EVENT_PENDING.store(false, Ordering::Release);
                if !app.runtime.is_active() {
                    return LRESULT(0);
                }
                app.clear_foreground_process_cache();
                app.tick();
                let has_managed_mutes = app.has_managed_mutes();
                app.runtime
                    .schedule_managed_mute_foreground_retry(has_managed_mutes);
                app.reset_polling_timer();
                return LRESULT(0);
            }
            WM_PROCESS_SEARCH_RESULT_CHOSEN => {
                app.commit_process_result(wparam.0);
                return LRESULT(0);
            }
            WM_REFRESH_THEME_VISUALS => {
                drop(app);
                unsafe {
                    let _ = RedrawWindow(
                        Some(hwnd),
                        None,
                        None,
                        RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_UPDATENOW,
                    );
                }
                return LRESULT(0);
            }
            WM_SHOW_PROCESS_RESULTS => {
                let request = app
                    .process_picker
                    .as_ref()
                    .and_then(|picker| unsafe { picker.prepare_show_popup() });
                drop(app);
                if let Some(request) = request {
                    unsafe {
                        let _ = request.show();
                    }
                }
                return LRESULT(0);
            }
            WM_PAINT => {
                app.paint(hwnd);
                return LRESULT(0);
            }
            WM_ERASEBKGND if app.erase_background(HDC(wparam.0 as *mut c_void)) => {
                return LRESULT(1);
            }
            WM_SETTINGCHANGE | WM_THEMECHANGED => {
                app.refresh_system_theme();
            }
            WM_MEASUREITEM if app.measure_item(lparam) => {
                return LRESULT(1);
            }
            WM_DRAWITEM if app.draw_item(lparam) => {
                return LRESULT(1);
            }
            WM_SHOWWINDOW => {
                if wparam.0 != 0 {
                    app.refresh_processes_if_stale();
                } else {
                    app.hide_process_results();
                }
                return LRESULT(0);
            }
            WM_ACTIVATE if loword(wparam.0 as u32) as u32 == WA_INACTIVE => {
                let activated_window = HWND(lparam.0 as *mut c_void);
                if !app.is_process_results_window(activated_window) {
                    app.hide_process_results();
                }
            }
            WM_LBUTTONDOWN => {
                app.background_click();
                return LRESULT(0);
            }
            WM_CONTEXTMENU if HWND(wparam.0 as *mut c_void) == app.controls.target_list => {
                let request = app.prepare_target_context_menu(lparam);
                drop(app);
                if let Some(request) = request {
                    unsafe {
                        execute_main_window_action(
                            state,
                            MainWindowAction::ShowTargetContextMenu(request),
                        );
                    }
                }
                return LRESULT(0);
            }
            WM_GETMINMAXINFO if lparam.0 != 0 => {
                let info = unsafe { &mut *(lparam.0 as *mut MINMAXINFO) };
                apply_window_minmax_info(hwnd, info);
                return LRESULT(0);
            }
            WM_SIZING if lparam.0 != 0 => {
                let rect = unsafe { &mut *(lparam.0 as *mut RECT) };
                constrain_sizing_rect(hwnd, wparam.0, rect);
                return LRESULT(1);
            }
            WM_MOVE => {
                app.remember_window_placement();
                return LRESULT(0);
            }
            WM_ENTERSIZEMOVE => {
                app.hide_process_results();
                app.interactive_resize = true;
                return LRESULT(0);
            }
            WM_EXITSIZEMOVE => {
                app.interactive_resize = false;
                app.rescale_ui_to_window(true);
                app.save_window_placement();
                return LRESULT(0);
            }
            WM_CLOSE => {
                app.hide_process_results();
                if app.settings_window_open {
                    return LRESULT(0);
                }
                if app.config.hide_to_tray_on_close {
                    if app.prepare_hide_to_tray() {
                        drop(app);
                        unsafe {
                            execute_main_window_action(state, MainWindowAction::HideToTray);
                        }
                    }
                } else {
                    drop(app);
                    unsafe {
                        execute_main_window_action(state, MainWindowAction::Close);
                    }
                }
                return LRESULT(0);
            }
            WM_SIZE => {
                let finalize_visuals = !app.interactive_resize;
                app.rescale_ui_to_window(finalize_visuals);
                return LRESULT(0);
            }
            WM_CTLCOLORSTATIC => {
                return app.static_control_color(wparam, lparam);
            }
            WM_TRAY_ICON => {
                match lparam.0 as u32 {
                    WM_LBUTTONDBLCLK => {
                        drop(app);
                        show_main_window(hwnd);
                    }
                    WM_RBUTTONUP => {
                        let request = app.prepare_tray_menu();
                        drop(app);
                        unsafe {
                            execute_main_window_action(
                                state,
                                MainWindowAction::ShowTrayMenu(request),
                            );
                        }
                    }
                    _ => {}
                }
                return LRESULT(0);
            }
            WM_DESTROY => {
                drop(app);
                unsafe {
                    PostQuitMessage(0);
                }
                return LRESULT(0);
            }
            WM_NCDESTROY => unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            },
            _ => {}
        }
    }

    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}
