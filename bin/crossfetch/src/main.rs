use std::{env, process::Stdio};

use anyhow::Result;
use argh::FromArgs;
use utils::{stats::System, version};

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

    let system = System::new();

    println!("{:?}", system);
    println!("{:?}", system.os_release());

    Ok(())
}
