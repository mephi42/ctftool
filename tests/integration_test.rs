extern crate anyhow;
extern crate tempdir;

use std::process::Command;

use anyhow::Result;
use tempdir::TempDir;

#[test]
fn integration_test() -> Result<()> {
    let mut exe = std::env::current_exe()?;
    exe.pop();
    exe.pop();
    exe = exe.join("ctf");
    let workdir = TempDir::new("ctf")?;
    assert!(Command::new(exe)
        .args(&["init"])
        .current_dir(workdir.path())
        .spawn()?
        .wait()?
        .success());
    Ok(())
}
