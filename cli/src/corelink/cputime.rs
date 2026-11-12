//! A child's CPU clock, user plus kernel time of the whole process, every
//! thread: the witness watch.rs reads to tell a core that works from a
//! core that is wedged. None when the platform has no reader here or the
//! reading fails; the rule then counts the request's bytes alone
//! (watch.rs).
//! - Windows: `GetProcessTimes`, kernel + user in 100 ns units.
//! - Linux: `/proc/<pid>/stat` fields 14 + 15 (utime, stime), the thread
//!   group's, in clock ticks of `sysconf(_SC_CLK_TCK)` (proc(5)).
//!   `/proc/<pid>/schedstat` is not read: it is one task's scheduler
//!   record, the thread-group leader's, not the process's.
//! - macOS: `proc_pid_rusage` (`RUSAGE_INFO_V2`). `ri_user_time` and
//!   `ri_system_time` carry no unit in the `libc` crate's declaration
//!   (libc 0.2.189, src/unix/bsd/apple/mod.rs, `rusage_info_v2`), and
//!   Apple's headers were not readable from this machine: they are read as
//!   mach absolute-time units and scaled by `mach_timebase_info`, whose
//!   ratio is not one on Apple silicon (not read on a Mac here). The
//!   physical leg (tests/unit/corelink/cputime.rs: a child that burns on a
//!   thread of its own until its own process clock has advanced one second
//!   must read between 0.8 s and 1.5 s) is what holds the unit and the
//!   whole process, on each CI platform.

use std::process::Child;
use std::time::Duration;

pub(super) fn cpu_time(child: &Child) -> Option<Duration> {
    imp::cpu_time(child)
}

#[cfg(windows)]
mod imp {
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::FILETIME;
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
}

#[cfg(target_os = "linux")]
mod imp {
    use std::process::Child;
    use std::time::Duration;

    pub(super) fn cpu_time(child: &Child) -> Option<Duration> {
        let pid = child.id();
        let text = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        // the command name (field 2) may hold spaces and parentheses: the
        // fields are read after its last ')', where field 3 comes first
        let fields: Vec<&str> = text[text.rfind(')')? + 1..].split_whitespace().collect();
        let tick = |i: usize| fields.get(i)?.parse::<u64>().ok();
        let ticks = tick(14 - 3)? + tick(15 - 3)?;
        // SAFETY: sysconf reads a system constant and touches no memory.
        let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        let hz = u64::try_from(hz).ok().filter(|&h| h > 0)?;
        Some(Duration::from_nanos(
            ticks.saturating_mul(1_000_000_000) / hz,
        ))
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::process::Child;
    use std::time::Duration;

    #[allow(deprecated)] // libc points at the mach2 crate, which the tree does not carry
    pub(super) fn cpu_time(child: &Child) -> Option<Duration> {
        let pid = i32::try_from(child.id()).ok()?;
        // SAFETY: rusage_info_v2 is plain integers, valid zeroed; the
        // kernel writes at most its size for flavor RUSAGE_INFO_V2.
        let mut info: libc::rusage_info_v2 = unsafe { std::mem::zeroed() };
        let buffer = (&raw mut info).cast::<libc::rusage_info_t>();
        // SAFETY: `buffer` points at a live rusage_info_v2.
        if unsafe { libc::proc_pid_rusage(pid, libc::RUSAGE_INFO_V2, buffer) } != 0 {
            return None;
        }
        let mut base = libc::mach_timebase_info { numer: 0, denom: 0 };
        // SAFETY: the out-pointer is a local.
        if unsafe { libc::mach_timebase_info(&mut base) } != 0 || base.denom == 0 {
            return None;
        }
        let abs = u128::from(info.ri_user_time) + u128::from(info.ri_system_time);
        let ns = abs * u128::from(base.numer) / u128::from(base.denom);
        Some(Duration::from_nanos(u64::try_from(ns).ok()?))
    }
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
mod imp {
    pub(super) fn cpu_time(_: &std::process::Child) -> Option<std::time::Duration> {
        None
    }
}

#[cfg(test)]
#[path = "../../tests/unit/corelink/cputime.rs"]
mod tests;
