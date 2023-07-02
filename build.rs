use anyhow::{Ok, Result};
use substreams_ethereum::Abigen;

fn main() -> Result<(), anyhow::Error> {
    Abigen::new("grt", "abi/grt.json")?
        .generate()?
        .write_to_file("src/abi/grt.rs")?;

    Ok(())
}
