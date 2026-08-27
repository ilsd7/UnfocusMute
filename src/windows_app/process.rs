use crate::config::normalize_supported_process_name_utf16;
use std::mem::size_of;
use windows::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_FILES, HANDLE, HWND, WAIT_OBJECT_0,
    WAIT_TIMEOUT,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    QueryFullProcessImageNameW, WaitForSingleObject,
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

pub(crate) struct ProcessLifetime(OwnedHandle);

impl ProcessLifetime {
    pub(crate) fn open(pid: u32) -> Option<Self> {
        let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid).ok()? };
        Some(Self(OwnedHandle(handle)))
    }

    pub(crate) fn has_exited(&self) -> Option<bool> {
        match unsafe { WaitForSingleObject(self.0.raw(), 0) } {
            WAIT_OBJECT_0 => Some(true),
            WAIT_TIMEOUT => Some(false),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessRefreshOutcome {
    Changed,
    Unchanged,
    Failed,
}

pub(crate) struct SnapshotProcessRefreshResult {
    pub(crate) outcome: ProcessRefreshOutcome,
    pub(crate) failure_detail: Option<String>,
}

#[derive(Default)]
pub struct ProcessNameResolver {
    names: ProcessNameCache,
    snapshot_names: SnapshotProcessNameCache,
    unresolved_pids: ProcessIdCache,
}

impl ProcessNameResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&mut self, pid: u32) -> Option<&str> {
        self.name_with(pid, process_image_name, process_names_from_snapshot)
    }

    fn name_with(
        &mut self,
        pid: u32,
        load_process_name: impl FnOnce(u32) -> Option<String>,
        load_snapshot_names: impl FnOnce() -> Option<ProcessNameSnapshot>,
    ) -> Option<&str> {
        if self.unresolved_pids.contains(pid) {
            return None;
        }

        if let Ok(index) = self.names.lookup(pid) {
            return Some(self.names.get_at(index));
        }
        let name = load_process_name(pid).or_else(|| {
            self.snapshot_names
                .names(load_snapshot_names)
                .and_then(|names| names.get(pid).map(str::to_owned))
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
enum SnapshotProcessNameCache {
    #[default]
    Unloaded,
    Unavailable,
    Loaded(ProcessNameSnapshot),
}

impl SnapshotProcessNameCache {
    fn names(
        &mut self,
        load: impl FnOnce() -> Option<ProcessNameSnapshot>,
    ) -> Option<&ProcessNameSnapshot> {
        if matches!(self, Self::Unloaded) {
            *self = match load() {
                Some(names) => Self::Loaded(names),
                None => Self::Unavailable,
            };
        }

        match self {
            Self::Loaded(names) => Some(names),
            Self::Unloaded | Self::Unavailable => None,
        }
    }
}

#[derive(Default)]
struct ProcessNameSnapshot {
    names: Vec<(u32, String)>,
}

impl ProcessNameSnapshot {
    fn new(mut names: Vec<(u32, String)>) -> Self {
        names.sort_unstable_by_key(|(pid, _)| *pid);
        names.dedup_by_key(|(pid, _)| *pid);
        Self { names }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            names: Vec::with_capacity(capacity),
        }
    }

    fn push(&mut self, pid: u32, name: String) {
        self.names.push((pid, name));
    }

    fn get(&self, pid: u32) -> Option<&str> {
        self.names
            .binary_search_by_key(&pid, |(cached_pid, _)| *cached_pid)
            .ok()
            .map(|index| self.names[index].1.as_str())
    }

    #[cfg(test)]
    fn empty() -> Self {
        Self::default()
    }

    #[cfg(test)]
    fn from_entries(entries: impl IntoIterator<Item = (u32, &'static str)>) -> Self {
        Self::new(
            entries
                .into_iter()
                .map(|(pid, name)| (pid, name.to_owned()))
                .collect(),
        )
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

#[derive(Default)]
struct ProcessNameCache {
    names: Vec<(u32, String)>,
}

impl ProcessNameCache {
    fn lookup(&self, pid: u32) -> Result<usize, usize> {
        self.names
            .binary_search_by_key(&pid, |(cached_pid, _)| *cached_pid)
    }

    fn get_at(&self, index: usize) -> &str {
        self.names[index].1.as_str()
    }

    fn insert_absent(&mut self, pid: u32, name: String) -> &str {
        let index = match self.lookup(pid) {
            Ok(index) => index,
            Err(index) => {
                self.names.insert(index, (pid, name));
                index
            }
        };
        self.names[index].1.as_str()
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

pub(crate) fn replace_processes(
    processes: &mut Vec<ProcessInfo>,
    scratch: &mut Vec<ProcessInfo>,
    collect: impl FnOnce(&mut Vec<ProcessInfo>) -> bool,
) -> ProcessRefreshOutcome {
    scratch.clear();
    let min_capacity = processes.capacity().max(EXPECTED_PROCESS_COUNT);
    if scratch.capacity() < min_capacity {
        scratch.reserve_exact(min_capacity);
    }

    if !collect(scratch) {
        scratch.clear();
        return ProcessRefreshOutcome::Failed;
    }
    sort_dedup_processes(scratch);
    if *processes == *scratch {
        scratch.clear();
        return ProcessRefreshOutcome::Unchanged;
    }
    std::mem::swap(processes, scratch);
    scratch.clear();
    ProcessRefreshOutcome::Changed
}

pub(crate) fn refresh_snapshot_processes(
    processes: &mut Vec<ProcessInfo>,
    scratch: &mut Vec<ProcessInfo>,
) -> SnapshotProcessRefreshResult {
    let mut failure_detail = None;
    let outcome = replace_processes(processes, scratch, |next| {
        if let Err(error) = collect_process_snapshot(next) {
            failure_detail = Some(error.to_string());
            return false;
        }
        true
    });
    SnapshotProcessRefreshResult {
        outcome,
        failure_detail,
    }
}

fn collect_process_snapshot(processes: &mut Vec<ProcessInfo>) -> Result<(), Error> {
    visit_process_snapshot(|pid, name| {
        processes.push(ProcessInfo { pid, name });
        true
    })
}

fn sort_dedup_processes(processes: &mut Vec<ProcessInfo>) {
    processes.sort_unstable_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.pid.cmp(&right.pid))
    });
    processes.dedup();
}

fn process_names_from_snapshot() -> Option<ProcessNameSnapshot> {
    let mut processes = ProcessNameSnapshot::with_capacity(EXPECTED_PROCESS_COUNT);
    if visit_process_snapshot(|pid, name| {
        processes.push(pid, name);
        true
    })
    .is_ok()
    {
        Some(ProcessNameSnapshot::new(processes.names))
    } else {
        None
    }
}

fn visit_process_snapshot(mut visit: impl FnMut(u32, String) -> bool) -> Result<(), Error> {
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

fn visit_process_snapshot_entries(
    mut visit: impl FnMut(&PROCESSENTRY32W) -> bool,
) -> Result<(), Error> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        let snapshot = OwnedHandle(snapshot);

        let mut entry = PROCESSENTRY32W {
            dwSize: u32::try_from(size_of::<PROCESSENTRY32W>())
                .expect("PROCESSENTRY32W size fits in u32"),
            ..Default::default()
        };

        Process32FirstW(snapshot.raw(), &mut entry)?;

        loop {
            if !visit(&entry) {
                break;
            }

            match Process32NextW(snapshot.raw(), &mut entry) {
                Ok(()) => continue,
                Err(error) if process_snapshot_finished(error.code()) => break,
                Err(error) => return Err(error),
            }
        }

        Ok(())
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
    let _ = visit_process_snapshot_entries(|entry| {
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
    fn current_process_lifetime_is_active() {
        let lifetime = ProcessLifetime::open(std::process::id())
            .expect("the current process must be open for synchronization");

        assert_eq!(lifetime.has_exited(), Some(false));
    }

    #[test]
    fn process_lifetime_detects_exit_without_pid_reuse() {
        let mut child = std::process::Command::new("cmd.exe")
            .args(["/D", "/C", "ping -n 2 127.0.0.1 >NUL"])
            .spawn()
            .expect("start short-lived child process");
        let lifetime = ProcessLifetime::open(child.id())
            .expect("the child process must be open for synchronization");

        assert_eq!(lifetime.has_exited(), Some(false));
        assert!(child.wait().expect("wait for child process").success());
        assert_eq!(lifetime.has_exited(), Some(true));
    }

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
    fn process_name_cache_keeps_sorted_lookup() {
        let mut cache = ProcessNameCache::default();
        for (pid, name) in [
            (30, "third.exe"),
            (10, "first.exe"),
            (20, "second.exe"),
            (40, "fourth.exe"),
        ] {
            assert_eq!(cache.insert_absent(pid, name.to_owned()), name);
        }

        assert_eq!(
            cache
                .names
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
        assert_eq!(cache.lookup(20), Ok(1));
        assert_eq!(cache.get_at(1), "second.exe");
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
        let mut scratch = Vec::new();

        assert_eq!(
            replace_processes(&mut processes, &mut scratch, |_| false),
            ProcessRefreshOutcome::Failed
        );
        assert_eq!(
            processes,
            [ProcessInfo {
                pid: 7,
                name: "old.exe".to_owned()
            }]
        );
        assert!(scratch.is_empty());
    }

    #[test]
    fn successful_process_refresh_replaces_and_sorts_processes() {
        let mut processes = vec![ProcessInfo {
            pid: 7,
            name: "old.exe".to_owned(),
        }];
        let mut scratch = Vec::new();

        assert_eq!(
            replace_processes(&mut processes, &mut scratch, |next| {
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
            }),
            ProcessRefreshOutcome::Changed
        );
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
        assert!(scratch.is_empty());
    }

    #[test]
    fn unchanged_process_refresh_reports_unchanged_without_replacing_processes() {
        let mut processes = vec![
            ProcessInfo {
                pid: 10,
                name: "alpha.exe".to_owned(),
            },
            ProcessInfo {
                pid: 20,
                name: "zeta.exe".to_owned(),
            },
        ];
        let mut scratch = Vec::new();

        assert_eq!(
            replace_processes(&mut processes, &mut scratch, |next| {
                next.push(ProcessInfo {
                    pid: 20,
                    name: "zeta.exe".to_owned(),
                });
                next.push(ProcessInfo {
                    pid: 10,
                    name: "alpha.exe".to_owned(),
                });
                true
            }),
            ProcessRefreshOutcome::Unchanged
        );
        assert_eq!(
            processes,
            [
                ProcessInfo {
                    pid: 10,
                    name: "alpha.exe".to_owned()
                },
                ProcessInfo {
                    pid: 20,
                    name: "zeta.exe".to_owned()
                }
            ]
        );
        assert!(scratch.is_empty());
    }

    #[test]
    fn process_refresh_reserves_scratch_for_existing_capacity_before_collecting() {
        let mut processes = Vec::with_capacity(EXPECTED_PROCESS_COUNT + 16);
        let mut scratch = Vec::with_capacity(1);

        assert_eq!(
            replace_processes(&mut processes, &mut scratch, |next| {
                assert!(next.capacity() >= EXPECTED_PROCESS_COUNT + 16);
                true
            }),
            ProcessRefreshOutcome::Unchanged
        );
    }

    #[test]
    fn process_refresh_deduplicates_collected_entries() {
        let mut processes = Vec::new();
        let mut scratch = Vec::new();

        assert_eq!(
            replace_processes(&mut processes, &mut scratch, |next| {
                next.push(ProcessInfo {
                    pid: 20,
                    name: "alpha.exe".to_owned(),
                });
                next.push(ProcessInfo {
                    pid: 20,
                    name: "alpha.exe".to_owned(),
                });
                true
            }),
            ProcessRefreshOutcome::Changed
        );
        assert_eq!(
            processes,
            [ProcessInfo {
                pid: 20,
                name: "alpha.exe".to_owned()
            }]
        );
        assert!(scratch.is_empty());
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
    fn process_name_snapshot_sorts_and_deduplicates_by_pid() {
        let snapshot = ProcessNameSnapshot::from_entries([
            (30, "third.exe"),
            (10, "first.exe"),
            (20, "second.exe"),
            (10, "first.exe"),
        ]);

        assert_eq!(snapshot.get(10), Some("first.exe"));
        assert_eq!(snapshot.get(20), Some("second.exe"));
        assert_eq!(snapshot.get(99), None);
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
                    Some(ProcessNameSnapshot::empty())
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
                    Some(ProcessNameSnapshot::from_entries([(42, "snapshot.exe")]))
                },
            ),
            None
        );

        assert_eq!(image_attempts, 1);
        assert_eq!(snapshot_attempts, 1);
    }

    #[test]
    fn process_name_resolver_skips_snapshot_when_direct_lookup_succeeds() {
        let mut resolver = ProcessNameResolver::new();
        let mut snapshot_attempts = 0;

        assert_eq!(
            resolver.name_with(
                42,
                |_| Some("game.exe".to_owned()),
                || {
                    snapshot_attempts += 1;
                    Some(ProcessNameSnapshot::from_entries([(42, "snapshot.exe")]))
                },
            ),
            Some("game.exe")
        );
        assert_eq!(
            resolver.name_with(
                42,
                |_| None,
                || {
                    snapshot_attempts += 1;
                    Some(ProcessNameSnapshot::empty())
                },
            ),
            Some("game.exe")
        );

        assert_eq!(snapshot_attempts, 0);
    }

    #[test]
    fn process_name_resolver_does_not_retry_failed_snapshot_load() {
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
                    None
                },
            ),
            None
        );
        assert_eq!(
            resolver.name_with(
                7,
                |_| {
                    image_attempts += 1;
                    None
                },
                || {
                    snapshot_attempts += 1;
                    Some(ProcessNameSnapshot::from_entries([(7, "snapshot.exe")]))
                },
            ),
            None
        );

        assert_eq!(image_attempts, 2);
        assert_eq!(snapshot_attempts, 1);
    }

    #[test]
    fn process_name_resolver_caches_successful_snapshot_fallback() {
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
                    Some(ProcessNameSnapshot::from_entries([(42, "game.exe")]))
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
                    Some(ProcessNameSnapshot::empty())
                },
            ),
            Some("game.exe")
        );

        assert_eq!(image_attempts, 1);
        assert_eq!(snapshot_attempts, 1);
    }

    fn wide_null_terminated(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }
}
