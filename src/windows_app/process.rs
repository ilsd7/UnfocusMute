use crate::config::normalize_process_name_utf16;
use std::collections::HashMap;
use std::mem::size_of;
use windows::Win32::Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, HANDLE, HWND};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use windows::core::{Error, HRESULT, PWSTR};

const EXPECTED_PROCESS_COUNT: usize = 128;
const PROCESS_IMAGE_BUFFER_LEN: usize = 1024;
const MAX_PROCESS_IMAGE_BUFFER_LEN: usize = 32_768;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

#[derive(Default)]
pub struct ProcessNameResolver {
    names: ProcessNameCache,
    snapshot_names: Option<HashMap<u32, String>>,
    prefer_snapshot: bool,
}

impl ProcessNameResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot_first() -> Self {
        Self {
            prefer_snapshot: true,
            ..Self::default()
        }
    }

    pub fn name(&mut self, pid: u32) -> Option<&str> {
        if self.prefer_snapshot {
            {
                let snapshot_names = self
                    .snapshot_names
                    .get_or_insert_with(process_names_from_snapshot);
                if let Some(name) = snapshot_names.get(&pid) {
                    return Some(name);
                }
            }

            return self.names.get_or_insert_with(pid, process_image_name);
        }

        if let Some(position) = self.names.position(pid) {
            return Some(self.names.name_at(position));
        }
        let name = process_image_name(pid).or_else(|| {
            let names = self
                .snapshot_names
                .get_or_insert_with(process_names_from_snapshot);
            names.get(&pid).cloned()
        })?;
        Some(self.names.insert(pid, name))
    }
}

#[derive(Clone, Copy)]
enum ProcessNameCachePosition {
    One,
    TwoFirst,
    TwoSecond,
    Many(usize),
}

#[derive(Default)]
enum ProcessNameCache {
    #[default]
    Empty,
    One {
        pid: u32,
        name: String,
    },
    Two {
        first_pid: u32,
        first_name: String,
        second_pid: u32,
        second_name: String,
    },
    Many {
        names: Vec<(u32, String)>,
    },
}

impl ProcessNameCache {
    fn position(&self, pid: u32) -> Option<ProcessNameCachePosition> {
        match self {
            Self::Empty => None,
            Self::One {
                pid: cached_pid, ..
            } => (*cached_pid == pid).then_some(ProcessNameCachePosition::One),
            Self::Two {
                first_pid,
                second_pid,
                ..
            } => {
                if *first_pid == pid {
                    Some(ProcessNameCachePosition::TwoFirst)
                } else if *second_pid == pid {
                    Some(ProcessNameCachePosition::TwoSecond)
                } else {
                    None
                }
            }
            Self::Many { names } => names
                .iter()
                .position(|(cached_pid, _)| *cached_pid == pid)
                .map(ProcessNameCachePosition::Many),
        }
    }

    fn name_at(&self, position: ProcessNameCachePosition) -> &str {
        match (self, position) {
            (Self::One { name, .. }, ProcessNameCachePosition::One) => name,
            (Self::Two { first_name, .. }, ProcessNameCachePosition::TwoFirst) => first_name,
            (Self::Two { second_name, .. }, ProcessNameCachePosition::TwoSecond) => second_name,
            (Self::Many { names }, ProcessNameCachePosition::Many(index)) => &names[index].1,
            _ => unreachable!("process name cache position must match cache variant"),
        }
    }

    fn get_or_insert_with(
        &mut self,
        pid: u32,
        load: impl FnOnce(u32) -> Option<String>,
    ) -> Option<&str> {
        if let Some(position) = self.position(pid) {
            return Some(self.name_at(position));
        }

        Some(self.insert(pid, load(pid)?))
    }

    fn insert(&mut self, pid: u32, name: String) -> &str {
        *self = match std::mem::take(self) {
            Self::Empty => Self::One { pid, name },
            Self::One {
                pid: first_pid,
                name: first_name,
            } => Self::Two {
                first_pid,
                first_name,
                second_pid: pid,
                second_name: name,
            },
            Self::Two {
                first_pid,
                first_name,
                second_pid,
                second_name,
            } => {
                let mut names = Vec::with_capacity(4);
                names.push((first_pid, first_name));
                names.push((second_pid, second_name));
                names.push((pid, name));
                Self::Many { names }
            }
            Self::Many { mut names } => {
                names.push((pid, name));
                Self::Many { names }
            }
        };

        match self {
            Self::One { name, .. } => name,
            Self::Two { second_name, .. } => second_name,
            Self::Many { names } => &names[names.len() - 1].1,
            Self::Empty => unreachable!("insert always stores a process name"),
        }
    }
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

pub fn refresh_running_processes(processes: &mut Vec<ProcessInfo>) {
    processes.clear();
    if processes.capacity() < EXPECTED_PROCESS_COUNT {
        processes.reserve(EXPECTED_PROCESS_COUNT);
    }
    visit_process_snapshot(|pid, name| {
        processes.push(ProcessInfo { pid, name });
        true
    });
    if processes.len() > 1 {
        processes.sort_unstable_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.pid.cmp(&right.pid))
        });
    }
}

fn process_names_from_snapshot() -> HashMap<u32, String> {
    let mut processes = HashMap::with_capacity(EXPECTED_PROCESS_COUNT);
    visit_process_snapshot(|pid, name| {
        processes.insert(pid, name);
        true
    });
    processes
}

fn visit_process_snapshot(mut visit: impl FnMut(u32, String) -> bool) {
    visit_process_snapshot_entries(|entry| {
        if entry.th32ProcessID == 0 {
            return true;
        }
        let Some(name) = normalize_process_name_utf16(&entry.szExeFile) else {
            return true;
        };
        visit(entry.th32ProcessID, name)
    });
}

fn visit_process_snapshot_entries(mut visit: impl FnMut(&PROCESSENTRY32W) -> bool) {
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return;
        };
        let snapshot = OwnedHandle(snapshot);

        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot.raw(), &mut entry).is_ok() {
            loop {
                if !visit(&entry) {
                    break;
                }

                if Process32NextW(snapshot.raw(), &mut entry).is_err() {
                    break;
                }
            }
        }
    }
}

pub fn process_name(pid: u32) -> Option<String> {
    process_image_name(pid).or_else(|| process_name_from_snapshot(pid))
}

fn process_image_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OwnedHandle(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?);
        query_process_image_name(handle.raw())
    }
}

fn process_name_from_snapshot(target_pid: u32) -> Option<String> {
    if target_pid == 0 {
        return None;
    }

    let mut result = None;
    visit_process_snapshot_entries(|entry| {
        if entry.th32ProcessID == target_pid {
            result = normalize_process_name_utf16(&entry.szExeFile);
            false
        } else {
            true
        }
    });
    result
}

struct OwnedHandle(HANDLE);

impl OwnedHandle {
    fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

unsafe fn query_process_image_name(handle: HANDLE) -> Option<String> {
    let mut buffer = [0u16; PROCESS_IMAGE_BUFFER_LEN];
    let len = match unsafe { query_process_image_name_into(handle, &mut buffer) } {
        Ok(len) => len,
        Err(error) if is_insufficient_buffer(&error) => {
            let mut buffer = vec![0u16; MAX_PROCESS_IMAGE_BUFFER_LEN];
            let len = unsafe { query_process_image_name_into(handle, &mut buffer) }.ok()?;
            return normalize_process_name_utf16(&buffer[..len]);
        }
        Err(_) => return None,
    };

    normalize_process_name_utf16(&buffer[..len])
}

unsafe fn query_process_image_name_into(
    handle: HANDLE,
    buffer: &mut [u16],
) -> Result<usize, Error> {
    let mut len = buffer.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        )
    }?;
    Ok(len as usize)
}

fn is_insufficient_buffer(error: &Error) -> bool {
    error.code() == HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_name_cache_retries_failed_loads() {
        let mut cache = ProcessNameCache::default();
        let mut attempts = 0;

        assert_eq!(
            cache.get_or_insert_with(7, |_| {
                attempts += 1;
                None
            }),
            None
        );
        assert_eq!(attempts, 1);

        assert_eq!(
            cache.get_or_insert_with(7, |_| {
                attempts += 1;
                Some("game.exe".to_owned())
            }),
            Some("game.exe")
        );
        assert_eq!(attempts, 2);

        assert_eq!(
            cache.get_or_insert_with(7, |_| {
                attempts += 1;
                Some("other.exe".to_owned())
            }),
            Some("game.exe")
        );
        assert_eq!(attempts, 2);
    }
}
