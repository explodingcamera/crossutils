use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Display, Formatter},
    path::Path,
};

use rustix::path::Arg;

#[derive(Debug)]
pub enum Kernel {
    NT,
    Linux,
    Darwin,
    NetBSD,
    OpenBSD,
    FreeBSD,
    Unknown,
}

#[derive(Debug)]
pub enum OS {
    // Linux
    EndeavourOS,
    Arch,
    Artix,
    Ubuntu,
    LinuxGeneric,

    // BSD
    NetBSD,
    OpenBSD,
    FreeBSD,

    // Windows
    Windows,
}

impl Display for Kernel {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for OS {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct OSRelase {
    pub os: Option<OS>,
    pub id: Option<String>,
    pub id_like: Option<String>,
    pub name: Option<String>,
    pub name_pretty: Option<String>,
    pub build: Option<String>,
    pub logo: Option<String>,
}

#[derive(Debug)]
pub struct System {
    pub kernel: Kernel,
    pub kernel_version: String,
    pub hostname: String,
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

impl System {
    pub fn new() -> Self {
        let x = rustix::process::uname();
        let kernel_version = x.version().to_owned().into_string().unwrap();
        let hostname = x.nodename().to_owned().into_string().unwrap();

        // let root = statfs(std::path::Path::new("/")).unwrap();
        // println!("{:?}", root);

        let sysname = x.sysname().to_str().unwrap();
        let kernel = match sysname {
            "Linux" => Kernel::Linux,
            _ if { sysname.starts_with("GNU") } => Kernel::Linux,
            "XNU's Not UNIX!" => Kernel::Darwin,
            "FreeBSD" => Kernel::FreeBSD,
            "NetBSD" => Kernel::NetBSD,
            "OpenBSD" => Kernel::OpenBSD,
            "The New Technology" => Kernel::NT,
            _ => Kernel::Unknown,
        };

        System {
            kernel,
            hostname,
            kernel_version,
        }
    }

    pub fn os_release(&self) -> OSRelase {
        match self.kernel {
            Kernel::Linux => {
                let mut release = OSRelase {
                    build: None,
                    id: None,
                    id_like: None,
                    name: None,
                    name_pretty: None,
                    logo: None,
                    os: Some(OS::LinuxGeneric),
                };

                if let Ok(contents) = crate::fs::readfile(Path::new("/etc/os-release")) {
                    let mut data: BTreeMap<String, String> = BTreeMap::new();
                    let text = contents
                        .to_string_lossy()
                        .trim_matches(char::from(0))
                        .to_string();

                    for line in text.lines() {
                        let mut parts = line.split('=');
                        if let Some(key) = parts.next() {
                            if let Some(value) = parts.next() {
                                data.insert(key.to_string(), value.to_string());
                            }
                        }
                    }

                    if let Some(id) = data.get("ID") {
                        release.id = Some(id.to_string());
                    }
                    if let Some(id_like) = data.get("ID_LIKE") {
                        release.id_like = Some(id_like.to_string());
                    }
                    if let Some(name) = data.get("NAME") {
                        release.name = Some(name.to_string());
                    }
                    if let Some(name_pretty) = data.get("PRETTY_NAME") {
                        release.name_pretty = Some(name_pretty.to_string());
                    }
                    if let Some(build) = data.get("BUILD_ID") {
                        release.build = Some(build.to_string());
                    }
                    if let Some(logo) = data.get("LOGO") {
                        release.logo = Some(logo.to_string());
                    }
                }
                release
            }
            _ => OSRelase {
                build: None,
                id: None,
                id_like: None,
                name: None,
                name_pretty: None,
                logo: None,
                os: None,
            },
        }
    }
}
