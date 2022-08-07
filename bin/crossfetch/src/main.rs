use std::{env, process::Stdio};

use anyhow::Result;
use argh::FromArgs;
use utils::{stats::get_os, version};

#[derive(FromArgs)]
/// A simple fetch command
struct Crossfetch {
    #[argh(switch, short = 'v')]
    /// output version information and exit
    version: bool,
}
fn main() -> Result<()> {
    let cfc: Crossfetch = argh::from_env();

    if cfc.version {
        println!(
            "{}",
            version(
                "crossfetch",
                env!("CARGO_PKG_VERSION"),
                &["Henry Gressmann"]
            )
        );
        return Ok(());
    }

    println!("{}", get_os());
    println!("{}", get_os());

    Ok(())
}
