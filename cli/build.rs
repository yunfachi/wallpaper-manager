use std::io::Error;
use std::path::Path;

use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Shell};

include!("src/opts.rs");

const COMPLETION_DIR: &str = "../completions";
const APP_NAME: &str = "wallpaper-manager";

fn build_shell_completion(outdir: &Path) -> Result<(), Error> {
    let mut app = Opts::command();
    let shells = Shell::value_variants();

    for shell in shells {
        generate_to(*shell, &mut app, APP_NAME, outdir)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/opts.rs");

    let outdir = completion_dir()?;
    build_shell_completion(&outdir)?;

    Ok(())
}

// https://github.com/LGFae/swww/blob/main/client/build.rs
fn completion_dir() -> std::io::Result<PathBuf> {
    let path = PathBuf::from(COMPLETION_DIR);
    if !path.is_dir() {
        std::fs::create_dir(&path)?;
    }
    Ok(path)
}
