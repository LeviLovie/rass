use anyhow::{Context, Result};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let mut loader = rdss::Loader::new("assets.rdss");
    loader.load().context("Failed to load binary")?;
    for file in loader.files() {
        if file.ends_with(".png") {
            let contents = loader
                .read_raw(&file)
                .context(format!("Failed to read {file}"))?;
            println!("{}: <{} bytes>", file, contents.len());
            continue;
        }

        let contents = loader
            .read(&file)
            .context(format!("Failed to read {file}"))?
            .replace("\n", "\\n");
        println!("{}: {}", file, contents);
    }

    Ok(())
}
