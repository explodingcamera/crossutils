use std::env;

use anyhow::Result;
use argh::FromArgs;
use utils::{
    rustix::fs::{cwd, openat, sendfile, statat, AtFlags, Mode, OFlags},
    version,
};

#[derive(FromArgs)]
/// A sometimes slightly faster alternative for 'cat'
struct Dog {
    #[argh(switch, short = 'v')]
    /// output version information and exit
    version: bool,

    #[argh(positional)]
    /// file
    file: Vec<String>,
}

fn main() -> Result<()> {
    let dog: Dog = argh::from_env();

    if dog.version {
        println!(
            "{}",
            version("dog", env!("CARGO_PKG_VERSION"), &["Henry Gressmann"])
        );
        return Ok(());
    }

    unsafe {
        let stdout = utils::rustix::io::stdout();

        for file in dog.file {
            let stats = statat(cwd(), file.clone(), AtFlags::empty())?;
            let in_fd = openat(cwd(), file, OFlags::RDONLY, Mode::empty())?;
            sendfile(stdout, in_fd, None, stats.st_size.try_into()?)?;
        }
    }

    Ok(())
}
