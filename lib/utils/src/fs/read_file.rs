use std::io::{IoSliceMut, Result};
use std::path::Path;

use libc::{c_void, size_t};
use rustix::fd::IntoRawFd;
use rustix::fs::{cwd, openat, Mode, OFlags};
use rustix::io::preadv;

use crate::cvt::cvt;

const LEN: usize = 4096;

pub fn readfile(path: &Path) -> Result<Vec<u8>> {
    let fd = openat(cwd(), path, OFlags::RDONLY, Mode::empty())?;

    let mut buf = [0u8; LEN];
    preadv(&fd, &mut [IoSliceMut::new(&mut buf)], 0)?;

    // let _size = cvt(unsafe {
    //     libc::read(
    //         fd.into_raw_fd(),
    //         buf.as_mut_ptr() as *mut c_void,
    //         buf.len() as size_t,
    //     )
    // })?;

    Ok(buf.to_vec())
}
