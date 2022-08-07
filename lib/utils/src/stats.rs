use std::{
    ffi::CString,
    fmt::{Display, Formatter},
    io,
    process::Command,
};

use crate::{
    cvt::cvt,
    spawn::{self, posix_spawn, CStringArray, ChildPipes},
};

pub fn get_hostname() -> io::Result<String> {
    let size = 255;
    let mut buffer = vec![0u8; size];
    cvt(unsafe { libc::gethostname(buffer.as_mut_ptr() as *mut libc::c_char, size) })?;
    Ok(String::from_utf8_lossy(&buffer).to_string())
}

#[derive(Debug)]
pub enum OS {
    Windows,
    Linux,
    Darwin,
    BSD,
    Unknown,
}

impl Display for OS {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub fn get_os() -> OS {
    let mut pipes = ChildPipes {
        stderr: (),
        stdout: (),
        stdin: (),
    };

    let mut envp = CStringArray::with_capacity(10);
    envp.push(CString::new("uname").unwrap());

    let uname = Command::new("false").output().unwrap();
    println!("{:?}", uname);

    let uname = Command::new("echo").arg("lol").output().unwrap();
    println!("{:?}", uname);

    // let _ = posix_spawn(&pipes, Some(&envp)).unwrap();
    let uname = Command::new("/sbin/uname").arg("-s").output();
    println!("{:?}", uname);

    let uname = match uname {
        Ok(uname) => String::from_utf8_lossy(&uname.stdout).to_string(),
        Err(_) => return OS::Windows,
    };

    match uname.as_str() {
        "Linux" => OS::Linux,
        _ if { uname.starts_with("GNU") } => OS::Linux,
        "Darwin" => OS::Darwin,
        "DragonFly" => OS::BSD,
        _ if { uname.ends_with("BSD") } => OS::BSD,
        _ => OS::Unknown,
    }
}
