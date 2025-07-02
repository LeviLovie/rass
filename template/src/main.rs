use anyhow::{Context, Result};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let mut loader = rdss::Loader::new("assets.rass");
    loader.load().context("Failed to load binary")?;
    for file in loader.files() {
        let contents = loader
            .read(&file)
            .context(format!("Failed to read {file}"))?;
        println!("{}: {}", file, contents.replace("\n", "\\n"));
    }

    Ok(())
}
