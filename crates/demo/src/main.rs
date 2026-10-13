use anyhow::{bail, Result};
use clap::Parser;
use demo::{execute, Args};

fn main() -> Result<()> {
    let checks = execute(&Args::parse())?;
    let failed: Vec<_> = checks.iter().filter(|c| !c.ok).map(|c| c.name).collect();
    if !failed.is_empty() {
        bail!("{} of {} checks failed: {}", failed.len(), checks.len(), failed.join(", "));
    }
    println!("All {} checks passed.", checks.len());
    Ok(())
}
