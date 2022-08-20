use std::{env, path::Path};

use anyhow::Result;
use argh::FromArgs;
use owo_colors::OwoColorize;
use utils::version;

#[derive(FromArgs)]
/// A simple alternative to gnu's `ls` command.
struct Lsw {
    #[argh(switch, short = 'v')]
    /// output version information and exit
    version: bool,
}

fn main() -> Result<()> {
    let lsw: Lsw = argh::from_env();

    if lsw.version {
        println!(
            "{}",
            version("lsw", env!("CARGO_PKG_VERSION"), &["Henry Gressmann"])
        );
        return Ok(());
    }

    let files = utils::fs::readdir(&env::current_dir()?)?;
    for file in files {
        let file = file?;
        let name = file.file_name();
        let name = name.to_string_lossy();
        let metadata = file.metadata()?;

        print!("{}  {:#?}", name.bold(), metadata);
    }
    let x = utils::process::exec("ls");
    println!("{x}");

    Ok(())
}

// pub fn read_dir(path: std::path::PathBuf) -> io::Result<nix::dir::Dir> {
//     let dir = nix::dir::Dir::open(&path, OFlag::O_DIRECTORY, Mode::S_IXUSR)?;
//     Ok(dir)
// }
