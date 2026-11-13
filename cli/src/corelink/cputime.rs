//! Two readings of a child process: its CPU clock, user plus kernel time of
//! the whole process, every thread - the witness watch.rs reads to tell a
//! core that works from a core that is wedged - and its page-fault count,
//! which the rule never reads: it rides the error a killing verdict writes,
//! so a core that waited a minute on the disk (paged out on a loaded
//! machine) can be told from one that is wedged. Each is None when the
//! platform has no reader here or the reading fails; the rule then counts
//! the request's bytes alone (watch.rs), the error says the count was
//! unreadable.
//! - Windows: `GetProcessTimes`, kernel + user in 100 ns units;
//!   `GetProcessMemoryInfo` `PageFaultCount`, soft and hard faults, a u32
//!   (so a difference is taken wrapping at 2^32).
//! - Linux: `/proc/<pid>/stat` (proc(5)), one parse read twice: fields
//!   14 + 15 (utime, stime), the thread group's, in clock ticks of
//!   `sysconf(_SC_CLK_TCK)`, and fields 10 + 12 (minflt, majflt).
//!   `/proc/<pid>/schedstat` is not read: it is one task's scheduler
//!   record, the thread-group leader's, not the process's.
//! - macOS: `proc_pid_rusage` (`RUSAGE_INFO_V2`), one read for both.
//!   `ri_user_time` and `ri_system_time` carry no unit in the `libc`
//!   crate's declaration (libc 0.2.189, src/unix/bsd/apple/mod.rs,
//!   `rusage_info_v2`), and Apple's headers were not readable from this
//!   machine: they are read as mach absolute-time units and scaled by
//!   `mach_timebase_info`, whose ratio is not one on Apple silicon (not
//!   read on a Mac here). The count is `ri_pageins`: page-ins only, not
//!   the faults a resident page answers, so the error says "page-ins".
//!   The physical leg (tests/unit/corelink/cputime.rs: a child that burns
//!   on a thread of its own until its own process clock has advanced one
//!   second must read between 0.8 s and 1.5 s) is what holds the unit and
//!   the whole process, on each CI platform.

use std::process::Child;
use std::time::Duration;

pub(super) fn cpu_time(child: &Child) -> Option<Duration> {
    imp::cpu_time(child)
}

/// The child's page-fault count so far (page-ins on macOS: `FAULTS`
/// names which).
pub(super) fn page_faults(child: &Child) -> Option<u64> {
    imp::page_faults(child)
}

/// What `page_faults` counts, in the platform's own word (macOS counts
/// page-ins only).
pub(super) const FAULTS: &str = if cfg!(target_os = "macos") {
    "page-ins"
} else {
    "page faults"
};

/// The count's width: Windows' `PageFaultCount` is a u32.
const BITS: u32 = if cfg!(windows) { 32 } else { 64 };

/// The child's page faults since `first` (a `page_faults` reading), None
/// when either reading is missing.
pub(super) fn faults_since(child: &Child, first: Option<u64>) -> Option<u64> {
    Some(faults_between(first?, page_faults(child)?))
}

/// The count's growth from `first` to `last`, wrapping at the reader's
/// width (32 bits on Windows).
fn faults_between(first: u64, last: u64) -> u64 {
    last.wrapping_sub(first) & (u64::MAX >> (64 - BITS))
}

#[cfg(windows)]
mod imp {
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::System::Threading::GetProcessTimes;

    pub(super) fn cpu_time(child: &Child) -> Option<Duration> {
        let zero = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let (mut created, mut exited, mut kernel, mut user) = (zero, zero, zero, zero);
        // SAFETY: the handle is the child's own process handle, open for
        // as long as the `Child` lives; the four out-pointers are locals.
        let ok = unsafe {
            GetProcessTimes(
                child.as_raw_handle(),
                &mut created,
                &mut exited,
                &mut kernel,
                &mut user,
            )
        };
        (ok != 0).then(|| hundreds(kernel) + hundreds(user))
    }

    fn hundreds(t: FILETIME) -> Duration {
        let n = (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime);
        Duration::from_nanos(n.saturating_mul(100))
    }

    pub(super) fn page_faults(child: &Child) -> Option<u64> {
        // SAFETY: PROCESS_MEMORY_COUNTERS is plain integers, valid zeroed.
        let mut counters: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
        let size = u32::try_from(std::mem::size_of::<PROCESS_MEMORY_COUNTERS>()).ok()?;
        // SAFETY: the handle is the child's own, open while the `Child`
        // lives; the out-pointer is a local of `size` bytes.
        let ok = unsafe { GetProcessMemoryInfo(child.as_raw_handle(), &mut counters, size) };
        (ok != 0).then(|| u64::from(counters.PageFaultCount))
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use std::process::Child;
    use std::time::Duration;

    /// The fields of `/proc/<pid>/stat` after the command name: it (field
    /// 2) may hold spaces and parentheses, so they are read after its last
    /// ')', where field 3 comes first.
    fn stat(child: &Child) -> Option<Vec<String>> {
        let text = std::fs::read_to_string(format!("/proc/{}/stat", child.id())).ok()?;
        let after = &text[text.rfind(')')? + 1..];
        Some(after.split_whitespace().map(String::from).collect())
    }

    /// proc(5)'s field `n`, a count.
    fn field(fields: &[String], n: usize) -> Option<u64> {
        fields.get(n - 3)?.parse().ok()
    }

    pub(super) fn cpu_time(child: &Child) -> Option<Duration> {
        let fields = stat(child)?;
        let ticks = field(&fields, 14)? + field(&fields, 15)?;
        // SAFETY: sysconf reads a system constant and touches no memory.
        let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        let hz = u64::try_from(hz).ok().filter(|&h| h > 0)?;
        Some(Duration::from_nanos(
            ticks.saturating_mul(1_000_000_000) / hz,
        ))
    }

    pub(super) fn page_faults(child: &Child) -> Option<u64> {
        let fields = stat(child)?;
        Some(field(&fields, 10)? + field(&fields, 12)?)
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::process::Child;
    use std::time::Duration;

    fn rusage(child: &Child) -> Option<libc::rusage_info_v2> {
        let pid = i32::try_from(child.id()).ok()?;
        // SAFETY: rusage_info_v2 is plain integers, valid zeroed; the
        // kernel writes at most its size for flavor RUSAGE_INFO_V2.
        let mut info: libc::rusage_info_v2 = unsafe { std::mem::zeroed() };
        let buffer = (&raw mut info).cast::<libc::rusage_info_t>();
        // SAFETY: `buffer` points at a live rusage_info_v2.
        let rc = unsafe { libc::proc_pid_rusage(pid, libc::RUSAGE_INFO_V2, buffer) };
        (rc == 0).then_some(info)
    }

    #[allow(deprecated)] // libc points at the mach2 crate, which the tree does not carry
    pub(super) fn cpu_time(child: &Child) -> Option<Duration> {
        let info = rusage(child)?;
        let mut base = libc::mach_timebase_info { numer: 0, denom: 0 };
        // SAFETY: the out-pointer is a local.
        if unsafe { libc::mach_timebase_info(&mut base) } != 0 || base.denom == 0 {
            return None;
        }
        let abs = u128::from(info.ri_user_time) + u128::from(info.ri_system_time);
        let ns = abs * u128::from(base.numer) / u128::from(base.denom);
        Some(Duration::from_nanos(u64::try_from(ns).ok()?))
    }

    pub(super) fn page_faults(child: &Child) -> Option<u64> {
        Some(rusage(child)?.ri_pageins)
    }
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
mod imp {
    pub(super) fn cpu_time(_: &std::process::Child) -> Option<std::time::Duration> {
        None
    }

    pub(super) fn page_faults(_: &std::process::Child) -> Option<u64> {
        None
    }
}

#[cfg(test)]
#[path = "../../tests/unit/corelink/cputime.rs"]
mod tests;
