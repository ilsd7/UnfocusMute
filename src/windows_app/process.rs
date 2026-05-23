use crate::config::normalize_supported_process_name_utf16;
use std::collections::HashMap;
use std::mem::size_of;
use windows::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_FILES, HANDLE, HWND,
};
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
    unresolved_pids: ProcessIdCache,
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
        self.name_with(pid, process_image_name, process_names_from_snapshot)
    }

    fn name_with(
        &mut self,
        pid: u32,
        load_process_name: impl FnOnce(u32) -> Option<String>,
        load_snapshot_names: impl FnOnce() -> HashMap<u32, String>,
    ) -> Option<&str> {
        if self.unresolved_pids.contains(pid) {
            return None;
        }

        if self.prefer_snapshot {
            {
                let snapshot_names = self.snapshot_names.get_or_insert_with(load_snapshot_names);
                if let Some(name) = snapshot_names.get(&pid) {
                    return Some(name);
                }
            }

            return match self.names.get_or_insert_with(pid, load_process_name) {
                Some(name) => Some(name),
                None => {
                    self.unresolved_pids.insert(pid);
                    None
                }
            };
        }

        if let Some(lookup) = self.names.lookup(pid) {
            return Some(self.names.get_at(lookup));
        }
        let name = load_process_name(pid).or_else(|| {
            let names = self.snapshot_names.get_or_insert_with(load_snapshot_names);
            names.get(&pid).cloned()
        });
        match name {
            Some(name) => Some(self.names.insert_absent(pid, name)),
            None => {
                self.unresolved_pids.insert(pid);
                None
            }
        }
    }
}

#[derive(Default)]
struct ProcessIdCache {
    pids: Vec<u32>,
}

impl ProcessIdCache {
    fn contains(&self, pid: u32) -> bool {
        self.pids.binary_search(&pid).is_ok()
    }

    fn insert(&mut self, pid: u32) {
        if let Err(index) = self.pids.binary_search(&pid) {
            self.pids.insert(index, pid);
        }
    }
}

#[derive(Clone, Copy)]
enum ProcessNameCacheLookup {
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
    fn lookup(&self, pid: u32) -> Option<ProcessNameCacheLookup> {
        match self {
            Self::Empty => None,
            Self::One {
                pid: cached_pid, ..
            } => (*cached_pid == pid).then_some(ProcessNameCacheLookup::One),
            Self::Two {
                first_pid,
                second_pid,
                ..
            } => {
                if *first_pid == pid {
                    Some(ProcessNameCacheLookup::TwoFirst)
                } else if *second_pid == pid {
                    Some(ProcessNameCacheLookup::TwoSecond)
                } else {
                    None
                }
            }
            Self::Many { names } => names
                .binary_search_by_key(&pid, |(cached_pid, _)| *cached_pid)
                .ok()
                .map(ProcessNameCacheLookup::Many),
        }
    }

    fn get_at(&self, lookup: ProcessNameCacheLookup) -> &str {
        match (self, lookup) {
            (Self::One { name, .. }, ProcessNameCacheLookup::One) => name,
            (Self::Two { first_name, .. }, ProcessNameCacheLookup::TwoFirst) => first_name,
            (Self::Two { second_name, .. }, ProcessNameCacheLookup::TwoSecond) => second_name,
            (Self::Many { names }, ProcessNameCacheLookup::Many(index)) => &names[index].1,
            _ => unreachable!("process name cache lookup must match cache storage"),
        }
    }

    fn get_or_insert_with(
        &mut self,
        pid: u32,
        load: impl FnOnce(u32) -> Option<String>,
    ) -> Option<&str> {
        if let Some(lookup) = self.lookup(pid) {
            return Some(self.get_at(lookup));
        }

        Some(self.insert_absent(pid, load(pid)?))
    }

    fn insert_absent(&mut self, pid: u32, name: String) -> &str {
        debug_assert!(self.lookup(pid).is_none());

        let lookup = match std::mem::take(self) {
            Self::Empty => {
                *self = Self::One { pid, name };
                ProcessNameCacheLookup::One
            }
            Self::One {
                pid: first_pid,
                name: first_name,
            } => {
                *self = Self::Two {
                    first_pid,
                    first_name,
                    second_pid: pid,
                    second_name: name,
                };
                ProcessNameCacheLookup::TwoSecond
            }
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
                names.sort_unstable_by_key(|(cached_pid, _)| *cached_pid);
                let index = insertion_index_by_pid(&names, pid);
                debug_assert_eq!(names[index].0, pid);
                *self = Self::Many { names };
                ProcessNameCacheLookup::Many(index)
            }
            Self::Many { mut names } => {
                let index = insertion_index_by_pid(&names, pid);
                debug_assert!(
                    names
                        .get(index)
                        .is_none_or(|(cached_pid, _)| *cached_pid != pid)
                );
                names.insert(index, (pid, name));
                *self = Self::Many { names };
                ProcessNameCacheLookup::Many(index)
            }
        };

        self.get_at(lookup)
    }
}

fn insertion_index_by_pid(names: &[(u32, String)], pid: u32) -> usize {
    names.partition_point(|(cached_pid, _)| *cached_pid < pid)
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

pub fn refresh_running_processes(processes: &mut Vec<ProcessInfo>) -> bool {
    replace_running_processes(processes, collect_running_processes)
}

fn replace_running_processes(
    processes: &mut Vec<ProcessInfo>,
    collect: impl FnOnce(&mut Vec<ProcessInfo>) -> bool,
) -> bool {
    let mut next = Vec::with_capacity(processes.capacity().max(EXPECTED_PROCESS_COUNT));
    if !collect(&mut next) {
        return false;
    }
    sort_running_processes(&mut next);
    *processes = next;
    true
}

fn collect_running_processes(processes: &mut Vec<ProcessInfo>) -> bool {
    visit_process_snapshot(|pid, name| {
        processes.push(ProcessInfo { pid, name });
        true
    })
}

fn sort_running_processes(processes: &mut [ProcessInfo]) {
    processes.sort_unstable_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.pid.cmp(&right.pid))
    });
}

fn process_names_from_snapshot() -> HashMap<u32, String> {
    let mut processes = HashMap::with_capacity(EXPECTED_PROCESS_COUNT);
    let _ = visit_process_snapshot(|pid, name| {
        processes.insert(pid, name);
        true
    });
    processes
}

fn visit_process_snapshot(mut visit: impl FnMut(u32, String) -> bool) -> bool {
    visit_process_snapshot_entries(|entry| {
        if entry.th32ProcessID == 0 {
            return true;
        }
        let Some(name) = normalize_supported_process_name_utf16(&entry.szExeFile) else {
            return true;
        };
        visit(entry.th32ProcessID, name)
    })
}

fn visit_process_snapshot_entries(mut visit: impl FnMut(&PROCESSENTRY32W) -> bool) -> bool {
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return false;
        };
        let snapshot = OwnedHandle(snapshot);

        let mut entry = PROCESSENTRY32W {
            dwSize: u32::try_from(size_of::<PROCESSENTRY32W>())
                .expect("PROCESSENTRY32W size fits in u32"),
            ..Default::default()
        };

        if Process32FirstW(snapshot.raw(), &mut entry).is_err() {
            return false;
        }

        loop {
            if !visit(&entry) {
                break;
            }

            match Process32NextW(snapshot.raw(), &mut entry) {
                Ok(()) => continue,
                Err(error) if process_snapshot_finished(error.code()) => break,
                Err(_) => return false,
            }
        }

        true
    }
}

fn process_snapshot_finished(error_code: HRESULT) -> bool {
    error_code == HRESULT::from_win32(ERROR_NO_MORE_FILES.0)
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
            result = normalize_supported_process_name_utf16(&entry.szExeFile);
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
            return normalize_supported_process_name_utf16(&buffer[..len]);
        }
        Err(_) => return None,
    };

    normalize_supported_process_name_utf16(&buffer[..len])
}

unsafe fn query_process_image_name_into(
    handle: HANDLE,
    buffer: &mut [u16],
) -> Result<usize, Error> {
    let mut len = u32::try_from(buffer.len()).expect("process image buffer length fits in u32");
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
    fn supported_process_names_require_exe_suffix() {
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated("Game.EXE")),
            Some("game.exe".to_owned())
        );
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated("System")),
            None
        );
        assert_eq!(
            normalize_supported_process_name_utf16(&wide_null_terminated("NUL.EXE")),
            None
        );
    }

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

    #[test]
    fn process_name_cache_uses_sorted_many_lookup() {
        let mut cache = ProcessNameCache::default();
        for (pid, name) in [
            (30, "third.exe"),
            (10, "first.exe"),
            (20, "second.exe"),
            (40, "fourth.exe"),
        ] {
            assert_eq!(
                cache.get_or_insert_with(pid, |_| Some(name.to_owned())),
                Some(name)
            );
        }

        let ProcessNameCache::Many { names } = &cache else {
            panic!("four cached entries should use many-cache storage");
        };
        assert_eq!(
            names
                .iter()
                .map(|(pid, name)| (*pid, name.as_str()))
                .collect::<Vec<_>>(),
            [
                (10, "first.exe"),
                (20, "second.exe"),
                (30, "third.exe"),
                (40, "fourth.exe")
            ]
        );

        let mut attempts = 0;
        assert_eq!(
            cache.get_or_insert_with(20, |_| {
                attempts += 1;
                Some("other.exe".to_owned())
            }),
            Some("second.exe")
        );
        assert_eq!(attempts, 0);
    }

    #[test]
    fn process_id_cache_keeps_sorted_unique_pids() {
        let mut cache = ProcessIdCache::default();
        for pid in [30, 10, 20, 10] {
            cache.insert(pid);
        }

        assert_eq!(cache.pids, [10, 20, 30]);
        assert!(cache.contains(20));
        assert!(!cache.contains(40));
    }

    #[test]
    fn failed_process_refresh_keeps_existing_processes() {
        let mut processes = vec![ProcessInfo {
            pid: 7,
            name: "old.exe".to_owned(),
        }];

        assert!(!replace_running_processes(&mut processes, |_| false));
        assert_eq!(
            processes,
            [ProcessInfo {
                pid: 7,
                name: "old.exe".to_owned()
            }]
        );
    }

    #[test]
    fn successful_process_refresh_replaces_and_sorts_processes() {
        let mut processes = vec![ProcessInfo {
            pid: 7,
            name: "old.exe".to_owned(),
        }];

        assert!(replace_running_processes(&mut processes, |next| {
            next.push(ProcessInfo {
                pid: 30,
                name: "zeta.exe".to_owned(),
            });
            next.push(ProcessInfo {
                pid: 20,
                name: "alpha.exe".to_owned(),
            });
            next.push(ProcessInfo {
                pid: 10,
                name: "alpha.exe".to_owned(),
            });
            true
        }));
        assert_eq!(
            processes,
            [
                ProcessInfo {
                    pid: 10,
                    name: "alpha.exe".to_owned()
                },
                ProcessInfo {
                    pid: 20,
                    name: "alpha.exe".to_owned()
                },
                ProcessInfo {
                    pid: 30,
                    name: "zeta.exe".to_owned()
                }
            ]
        );
    }

    #[test]
    fn process_snapshot_next_only_treats_no_more_files_as_finished() {
        assert!(process_snapshot_finished(HRESULT::from_win32(
            ERROR_NO_MORE_FILES.0
        )));
        assert!(!process_snapshot_finished(HRESULT::from_win32(
            ERROR_INSUFFICIENT_BUFFER.0
        )));
    }

    #[test]
    fn process_name_resolver_caches_direct_first_lookup_failures() {
        let mut resolver = ProcessNameResolver::new();
        let mut image_attempts = 0;
        let mut snapshot_attempts = 0;

        assert_eq!(
            resolver.name_with(
                42,
                |_| {
                    image_attempts += 1;
                    None
                },
                || {
                    snapshot_attempts += 1;
                    HashMap::new()
                },
            ),
            None
        );
        assert_eq!(
            resolver.name_with(
                42,
                |_| {
                    image_attempts += 1;
                    Some("late.exe".to_owned())
                },
                || {
                    snapshot_attempts += 1;
                    HashMap::from([(42, "snapshot.exe".to_owned())])
                },
            ),
            None
        );

        assert_eq!(image_attempts, 1);
        assert_eq!(snapshot_attempts, 1);
    }

    #[test]
    fn process_name_resolver_caches_snapshot_first_lookup_failures() {
        let mut resolver = ProcessNameResolver::snapshot_first();
        let mut image_attempts = 0;
        let mut snapshot_attempts = 0;

        assert_eq!(
            resolver.name_with(
                42,
                |_| {
                    image_attempts += 1;
                    None
                },
                || {
                    snapshot_attempts += 1;
                    HashMap::new()
                },
            ),
            None
        );
        assert_eq!(
            resolver.name_with(
                42,
                |_| {
                    image_attempts += 1;
                    Some("late.exe".to_owned())
                },
                || {
                    snapshot_attempts += 1;
                    HashMap::from([(42, "snapshot.exe".to_owned())])
                },
            ),
            None
        );

        assert_eq!(image_attempts, 1);
        assert_eq!(snapshot_attempts, 1);
    }

    #[test]
    fn process_name_resolver_keeps_successful_snapshot_lookup_reusable() {
        let mut resolver = ProcessNameResolver::snapshot_first();
        let mut image_attempts = 0;
        let mut snapshot_attempts = 0;

        assert_eq!(
            resolver.name_with(
                42,
                |_| {
                    image_attempts += 1;
                    None
                },
                || {
                    snapshot_attempts += 1;
                    HashMap::from([(42, "game.exe".to_owned())])
                },
            ),
            Some("game.exe")
        );
        assert_eq!(
            resolver.name_with(
                42,
                |_| {
                    image_attempts += 1;
                    None
                },
                || {
                    snapshot_attempts += 1;
                    HashMap::new()
                },
            ),
            Some("game.exe")
        );

        assert_eq!(image_attempts, 0);
        assert_eq!(snapshot_attempts, 1);
    }

    fn wide_null_terminated(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }
}
