mod clipboard;
mod cli;
mod storage;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
