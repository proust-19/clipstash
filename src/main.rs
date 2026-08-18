mod clipboard;
mod cli;
mod gui;
mod sensitive;
mod storage;


use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
