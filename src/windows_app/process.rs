use crate::config::normalize_process_name;
use std::mem::size_of;
use std::path::Path;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use windows::core::PWSTR;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
}

pub fn foreground_pid() -> Option<u32> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == HWND::default() {
            return None;
        }

        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32));
        if pid == 0 {
            return None;
        }

        Some(pid)
    }
}

pub fn running_processes() -> Vec<ProcessInfo> {
    let mut processes = Vec::new();

    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return processes;
        };

        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = utf16_array_to_string(&entry.szExeFile);
                if let Some(name) = normalize_process_name(&name) {
                    processes.push(ProcessInfo {
                        pid: entry.th32ProcessID,
                        name,
                        path: None,
                    });
                }

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
    }

    processes.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.pid.cmp(&right.pid))
    });
    processes
}

pub fn process_info(pid: u32) -> Option<ProcessInfo> {
    let path = process_path(pid);
    let name = path
        .as_deref()
        .and_then(|path| Path::new(path).file_name())
        .and_then(|name| name.to_str())
        .and_then(normalize_process_name)
        .or_else(|| process_name_from_snapshot(pid))?;

    Some(ProcessInfo { pid, name, path })
}

fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let result = query_process_path(handle);
        let _ = CloseHandle(handle);
        result
    }
}

unsafe fn query_process_path(handle: HANDLE) -> Option<String> {
    let mut buffer = vec![0u16; 32768];
    let mut len = buffer.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        )
        .ok()?;
    }
    Some(String::from_utf16_lossy(&buffer[..len as usize]))
}

fn process_name_from_snapshot(pid: u32) -> Option<String> {
    running_processes()
        .into_iter()
        .find(|process| process.pid == pid)
        .map(|process| process.name)
}

fn utf16_array_to_string(buffer: &[u16]) -> String {
    let len = buffer
        .iter()
        .position(|ch| *ch == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}
