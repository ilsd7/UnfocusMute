use crate::config::normalize_process_name_utf16;
use std::collections::{HashMap, hash_map::Entry};
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
    names: HashMap<u32, Option<String>>,
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
            let snapshot_names = self
                .snapshot_names
                .get_or_insert_with(process_names_from_snapshot);
            if let Some(name) = snapshot_names.get(&pid) {
                return Some(name);
            }

            return match self.names.entry(pid) {
                Entry::Occupied(entry) => entry.into_mut().as_deref(),
                Entry::Vacant(entry) => entry.insert(process_image_name(pid)).as_deref(),
            };
        }

        match self.names.entry(pid) {
            Entry::Occupied(entry) => entry.into_mut().as_deref(),
            Entry::Vacant(entry) => {
                let name = process_image_name(pid).or_else(|| {
                    let names = self
                        .snapshot_names
                        .get_or_insert_with(process_names_from_snapshot);
                    names.get(&pid).cloned()
                });
                entry.insert(name).as_deref()
            }
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
        processes.reserve(EXPECTED_PROCESS_COUNT - processes.capacity());
    }
    visit_process_snapshot(|pid, name| processes.push(ProcessInfo { pid, name }));
    processes.sort_unstable_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.pid.cmp(&right.pid))
    });
}

fn process_names_from_snapshot() -> HashMap<u32, String> {
    let mut processes = HashMap::with_capacity(EXPECTED_PROCESS_COUNT);
    visit_process_snapshot(|pid, name| {
        processes.insert(pid, name);
    });
    processes
}

fn visit_process_snapshot(mut visit: impl FnMut(u32, String)) {
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
                if entry.th32ProcessID != 0
                    && let Some(name) = normalize_process_name_utf16(&entry.szExeFile)
                {
                    visit(entry.th32ProcessID, name);
                }

                if Process32NextW(snapshot.raw(), &mut entry).is_err() {
                    break;
                }
            }
        }
    }
}

pub fn process_name(pid: u32) -> Option<String> {
    ProcessNameResolver::new().name(pid).map(str::to_owned)
}

fn process_image_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OwnedHandle(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?);
        query_process_image_name(handle.raw())
    }
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
