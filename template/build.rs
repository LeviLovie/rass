use anyhow::{Context, Result};

const ASSETS_DIR: &str = "assets";
const ASSETS_FILE: &str = "assets.rass";

fn main() -> Result<()> {
    let compiler = rdss::Compiler::builder()
        .from_sources(ASSETS_DIR)
        .save_to(ASSETS_FILE)
        .build()
        .context("Failed to build compiler")?;
    compiler.compile().context("Compilation failed")?;

    watch_dir(ASSETS_DIR);
    Ok(())
}

fn watch_dir(dir: impl Into<std::path::PathBuf>) {
    let entries = std::fs::read_dir(dir.into())
        .expect("Failed to read directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    for entry in entries {
        if entry.is_dir() {
            watch_dir(entry);
        } else if entry.extension().map_or(false, |ext| ext == "rass") {
            println!("cargo:rerun-if-changed={}", entry.display());
        }
    }
}
