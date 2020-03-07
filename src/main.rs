extern crate ctftool;
use anyhow::Result;

fn main() -> Result<()> {
    ctftool::main_sync(std::env::args(), std::env::current_dir()?)
}
