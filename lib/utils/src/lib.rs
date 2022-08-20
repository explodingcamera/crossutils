#![feature(rustc_private)]
mod cvt;
pub mod fs;
pub mod spawn;
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
