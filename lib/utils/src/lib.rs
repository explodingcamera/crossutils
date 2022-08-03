#![feature(rustc_private)]
pub mod fs;

pub fn version(package: &str, version: &str, authors: &[&str]) -> String {
    format!(
        "{package} (crossutils) version {version}\n\
        License ISC <https://opensource.org/licenses/ISC>\n\n\
        Written by {}",
        authors.join(", "),
    )
}
