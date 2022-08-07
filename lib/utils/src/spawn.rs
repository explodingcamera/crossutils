use std::{
    ffi::{CStr, CString},
    fs::File,
    io::{self},
    os::unix::prelude::{AsRawFd, OsStrExt},
    process::ChildStdout,
    process::{ChildStderr, Stdio},
    process::{ChildStdin, Command},
    ptr,
};

use libc::{c_char, sigaddset, sigemptyset};

use crate::cvt::cvt;

pub struct CStringArray {
    items: Vec<CString>,
    ptrs: Vec<*const c_char>,
}

impl CStringArray {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut result = CStringArray {
            items: Vec::with_capacity(capacity),
            ptrs: Vec::with_capacity(capacity + 1),
        };
        result.ptrs.push(ptr::null());
        result
    }
    pub fn push(&mut self, item: CString) {
        let l = self.ptrs.len();
        self.ptrs[l - 1] = item.as_ptr();
        self.ptrs.push(ptr::null());
        self.items.push(item);
    }
    pub fn as_ptr(&self) -> *const *const c_char {
        self.ptrs.as_ptr()
    }
}

pub struct ChildPipes {
    pub stdin: (),
    pub stdout: (),
    pub stderr: (),
}

pub fn posix_spawn(stdio: &ChildPipes, envp: Option<&CStringArray>) -> io::Result<()> {
    use std::mem::MaybeUninit;

    // if self.get_gid().is_some()
    //     || self.get_uid().is_some()
    //     || (self.env_saw_path() && !self.program_is_path())
    //     || !self.get_closures().is_empty()
    //     || self.get_groups().is_some()
    //     || self.get_create_pidfd()
    // {
    //     return Ok(None);
    // }

    // // Solaris, glibc 2.29+, and musl 1.24+ can set a new working directory,
    // // and maybe others will gain this non-POSIX function too. We'll check
    // // for this weak symbol as soon as it's needed, so we can return early
    // // otherwise to do a manual chdir before exec.
    // weak! {
    //     fn posix_spawn_file_actions_addchdir_np(
    //         *mut libc::posix_spawn_file_actions_t,
    //         *const libc::c_char
    //     ) -> libc::c_int
    // }
    // let addchdir = match self.get_cwd() {
    //     Some(cwd) => {
    //         match posix_spawn_file_actions_addchdir_np.get() {
    //             Some(f) => Some((f, cwd)),
    //             None => return Ok(None),
    //         }
    //     }
    //     None => None,
    // };
    let addchdir: Option<()> = None;
    // let pgroup = self.get_pgroup();

    // Safety: -1 indicates we don't have a pidfd.
    // let mut p = unsafe { Process::new(0, -1) };

    struct PosixSpawnFileActions<'a>(&'a mut MaybeUninit<libc::posix_spawn_file_actions_t>);

    impl Drop for PosixSpawnFileActions<'_> {
        fn drop(&mut self) {
            unsafe {
                libc::posix_spawn_file_actions_destroy(self.0.as_mut_ptr());
            }
        }
    }

    struct PosixSpawnattr<'a>(&'a mut MaybeUninit<libc::posix_spawnattr_t>);

    impl Drop for PosixSpawnattr<'_> {
        fn drop(&mut self) {
            unsafe {
                libc::posix_spawnattr_destroy(self.0.as_mut_ptr());
            }
        }
    }

    let x = Command::new("yes");
    let stdout = io::stdout();
    let stderr = io::stderr();

    unsafe {
        let mut attrs = MaybeUninit::uninit();
        cvt(libc::posix_spawnattr_init(attrs.as_mut_ptr()))?;
        let attrs = PosixSpawnattr(&mut attrs);

        let mut flags = 0;

        let mut file_actions = MaybeUninit::uninit();
        cvt(libc::posix_spawn_file_actions_init(
            file_actions.as_mut_ptr(),
        ))?;
        let file_actions = PosixSpawnFileActions(&mut file_actions);

        // TODO: STDIO
        // if let Some(fd) = stdio.stdin.fd() {
        //     cvt(libc::posix_spawn_file_actions_adddup2(
        //         file_actions.0.as_mut_ptr(),
        //         fd,
        //         libc::STDIN_FILENO,
        //     ))?;
        // }
        // if let Some(fd) = stdio.stdout.fd() {
        cvt(libc::posix_spawn_file_actions_adddup2(
            file_actions.0.as_mut_ptr(),
            // stdout.0.as_raw_fd(),
            stdout.as_raw_fd(),
            libc::STDOUT_FILENO,
        ))?;
        // }

        // if let Some(fd) = stdio.stderr.fd() {
        cvt(libc::posix_spawn_file_actions_adddup2(
            file_actions.0.as_mut_ptr(),
            stderr.as_raw_fd(),
            libc::STDERR_FILENO,
        ))?;
        // }

        // if let Some((f, cwd)) = addchdir {
        //     cvt(f(file_actions.0.as_mut_ptr(), cwd.as_ptr()))?;
        // }

        // if let Some(pgroup) = pgroup {
        //     flags |= libc::POSIX_SPAWN_SETPGROUP;
        //     cvt_nz(libc::posix_spawnattr_setpgroup(
        //         attrs.0.as_mut_ptr(),
        //         pgroup,
        //     ))?;
        // }

        let mut set = MaybeUninit::<libc::sigset_t>::uninit();
        cvt(sigemptyset(set.as_mut_ptr()))?;
        cvt(libc::posix_spawnattr_setsigmask(
            attrs.0.as_mut_ptr(),
            set.as_ptr(),
        ))?;

        cvt(sigaddset(set.as_mut_ptr(), libc::SIGPIPE))?;
        cvt(libc::posix_spawnattr_setsigdefault(
            attrs.0.as_mut_ptr(),
            set.as_ptr(),
        ))?;

        flags |= libc::POSIX_SPAWN_SETSIGDEF | libc::POSIX_SPAWN_SETSIGMASK;
        cvt(libc::posix_spawnattr_setflags(
            attrs.0.as_mut_ptr(),
            flags as _,
        ))?;

        // Make sure we synchronize access to the global `environ` resource
        // let _env_lock = sys::os::env_read_lock();
        let envp = envp.map(|c| c.as_ptr()).unwrap();

        let mut pid: i32 = -1;
        let program: CString = CString::new("uname").unwrap();
        let argv: CString = CString::new("").unwrap();

        println!("spawn");
        let mut envp_owned = Vec::new();
        for (key, value) in std::env::vars_os() {
            let mut env = key;
            env.push("=");
            env.push(&value);
            env.push("\0");
            envp_owned.push(env);
        }
        let mut envp = Vec::new();
        for var in &envp_owned {
            envp.push(var.as_bytes().as_ptr());
        }
        envp.push(std::ptr::null());

        let x = libc::posix_spawnp(
            &mut pid,
            program.as_c_str().as_ptr(),
            // self.get_program_cstr().as_ptr(),
            file_actions.0.as_ptr(),
            attrs.0.as_ptr(),
            argv.as_c_str().as_ptr() as *const _,
            envp.as_ptr() as _,
        );

        println!("pog {x}");
    }

    Ok(())
}
