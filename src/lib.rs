mod err;
mod sys;

pub mod caps;
pub mod prctl;

pub use caps::*;
pub use err::*;
pub use prctl::*;

#[inline]
unsafe fn raw_prctl(
    option: libc::c_int,
    arg2: libc::c_ulong,
    arg3: libc::c_ulong,
    arg4: libc::c_ulong,
    arg5: libc::c_ulong,
) -> Result<libc::c_int> {
    let res = libc::prctl(option, arg2, arg3, arg4, arg5);

    if res >= 0 {
        Ok(res)
    } else {
        Err(Error::last())
    }
}

#[inline]
unsafe fn raw_prctl_opt(
    option: libc::c_int,
    arg2: libc::c_ulong,
    arg3: libc::c_ulong,
    arg4: libc::c_ulong,
    arg5: libc::c_ulong,
) -> Option<libc::c_int> {
    let res = libc::prctl(option, arg2, arg3, arg4, arg5);

    if res >= 0 {
        Some(res)
    } else {
        None
    }
}
