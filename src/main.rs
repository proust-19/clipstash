mod clipboard;
mod cli;
mod gui;
mod storage;


use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
