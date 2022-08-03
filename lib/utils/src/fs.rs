use libc::{c_int, dirent};
use std::{
    ffi::{CStr, CString, OsStr, OsString},
    fmt, io,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
    sync::Arc,
};

struct dirent_min {
    d_ino: u64,
    d_type: u8,
}

pub struct DirEntry {
    dir: Arc<InnerReadDir>,
    entry: dirent_min,
    // We need to store an owned copy of the entry name on platforms that use
    // readdir() (not readdir_r()), because a) struct dirent may use a flexible
    // array to store the name, b) it lives only until the next readdir() call.
    name: CString,
}

impl DirEntry {
    fn name_bytes(&self) -> &[u8] {
        self.name.to_bytes()
    }

    pub fn file_name_os_str(&self) -> &OsStr {
        OsStr::from_bytes(self.name_bytes())
    }

    pub fn file_name(&self) -> OsString {
        self.file_name_os_str().to_os_string()
    }
}

struct Dir(*mut libc::DIR);

struct InnerReadDir {
    dirp: Dir,
    root: PathBuf,
}

pub struct ReadDir {
    inner: Arc<InnerReadDir>,
    #[cfg(not(any(
        target_os = "android",
        target_os = "linux",
        target_os = "solaris",
        target_os = "illumos",
        target_os = "fuchsia",
        target_os = "redox",
    )))]
    end_of_stream: bool,
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // This will only be called from std::fs::ReadDir, which will add a "ReadDir()" frame.
        // Thus the result will be e g 'ReadDir("/home")'
        fmt::Debug::fmt(&*self.inner.root, f)
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    #[cfg(any(
        target_os = "android",
        target_os = "linux",
        target_os = "solaris",
        target_os = "fuchsia",
        target_os = "redox",
        target_os = "illumos"
    ))]
    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        use libc::readdir;

        unsafe {
            loop {
                // As of POSIX.1-2017, readdir() is not required to be thread safe; only
                // readdir_r() is. However, readdir_r() cannot correctly handle platforms
                // with unlimited or variable NAME_MAX.  Many modern platforms guarantee
                // thread safety for readdir() as long an individual DIR* is not accessed
                // concurrently, which is sufficient for Rust.
                clear_errno();
                let entry_ptr = readdir(self.inner.dirp.0);
                if entry_ptr.is_null() {
                    // null can mean either the end is reached or an error occurred.
                    // So we had to clear errno beforehand to check for an error now.
                    return match errno() {
                        0 => None,
                        e => Some(Err(io::Error::from_raw_os_error(e))),
                    };
                }

                // Only d_reclen bytes of *entry_ptr are valid, so we can't just copy the
                // whole thing (#93384).  Instead, copy everything except the name.
                let mut copy: dirent = std::mem::zeroed();
                // Can't dereference entry_ptr, so use the local entry to get
                // offsetof(struct dirent, d_name)
                let copy_bytes = &mut copy as *mut _ as *mut u8;
                let copy_name = &mut copy.d_name as *mut _ as *mut u8;
                let name_offset = copy_name.offset_from(copy_bytes) as usize;
                let entry_bytes = entry_ptr as *const u8;
                let entry_name = entry_bytes.add(name_offset);
                std::ptr::copy_nonoverlapping(entry_bytes, copy_bytes, name_offset);

                let entry = dirent_min {
                    d_ino: copy.d_ino as u64,
                    #[cfg(not(any(target_os = "solaris", target_os = "illumos")))]
                    d_type: copy.d_type as u8,
                };

                let ret = DirEntry {
                    entry,
                    // d_name is guaranteed to be null-terminated.
                    name: CStr::from_ptr(entry_name as *const _).to_owned(),
                    dir: Arc::clone(&self.inner),
                };

                if ret.name_bytes() != b"." && ret.name_bytes() != b".." {
                    return Some(Ok(ret));
                }
            }
        }
    }
}

unsafe fn errno_location() -> *mut c_int {
    libc::__errno_location()
}

pub fn errno() -> i32 {
    unsafe { (*errno_location()) as i32 }
}

fn clear_errno() {
    // Safe because errno is a thread-local variable
    unsafe {
        *errno_location() = 0;
    }
}

fn cstr(path: &Path) -> io::Result<CString> {
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

pub fn readdir(p: &Path) -> io::Result<ReadDir> {
    let root = p.to_path_buf();
    let p = cstr(p)?;
    unsafe {
        let ptr = libc::opendir(p.as_ptr());
        if ptr.is_null() {
            Err(io::Error::last_os_error())
        } else {
            let inner = InnerReadDir {
                dirp: Dir(ptr),
                root,
            };
            Ok(ReadDir {
                inner: Arc::new(inner),
            })
        }
    }
}
