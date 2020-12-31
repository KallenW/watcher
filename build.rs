use anyhow::{bail, Result};

fn main() -> Result<()> {
    if !cfg!(any(windows, unix)) {
        bail!("Unsupported platform: {}", ::std::env::consts::OS);
    } else {
        Ok(())
    }
}
