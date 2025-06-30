use anyhow::{Context, Result};
use rass::prelude::*;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let compiler = Compiler::builder()
        .from_sources("assets")
        .save_to("assets.rass")
        .build()
        .context("Failed to build compiler")?;
    compiler.compile().context("Compilation failed")?;

    let mut loader = Loader::new("assets.rass");
    loader.load().context("Failed to load binary")?;
    for (file, _) in loader.files() {
        let content = loader
            .read(&file)
            .context(format!("Failed to read file {}", file))?
            .replace("\n", "\\n");
        println!("{}: {}", file, content);
    }

    Ok(())
}
