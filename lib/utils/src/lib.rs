#![feature(rustc_private)]
#![feature(is_some_with)]
#![feature(read_buf)]
#![feature(raw_os_nonzero)]

mod cvt;
pub mod fs;
pub mod process;
pub mod stats;

pub use rustix;

pub fn version(package: &str, version: &str, authors: &[&str]) -> String {
    format!(
        "{package} (crossutils) version {version}\n\
        License ISC <https://opensource.org/licenses/ISC>\n\n\
        Written by {}",
        authors.join(", "),
    )
}
