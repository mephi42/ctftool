extern crate anyhow;
extern crate tempdir;

use anyhow::Result;
use assert_cmd::Command;
use tempdir::TempDir;

struct WorkDir {
    temp_dir: TempDir,
}

impl WorkDir {
    fn new() -> Result<WorkDir> {
        let temp_dir = TempDir::new("ctf")?;
        Ok(WorkDir { temp_dir })
    }
}

fn command(work_dir: &WorkDir, args: &[&str]) -> Result<Command> {
    let mut command = Command::cargo_bin("ctf")?;
    command.args(args).current_dir(work_dir.temp_dir.path());
    Ok(command)
}

#[test]
fn test_init() -> Result<()> {
    let work_dir = WorkDir::new()?;
    command(&work_dir, &["init"])?.assert().success();
    command(&work_dir, &["init"])?.assert().failure();
    Ok(())
}

#[test]
fn test_remote() -> Result<()> {
    let work_dir = WorkDir::new()?;
    command(&work_dir, &["init"])?.assert().success();
    command(&work_dir, &["remote", "show"])?.assert().success();
    let url = "http://localhost.test";
    command(&work_dir, &["remote", "add", "origin", url])?
        .assert()
        .success();
    command(&work_dir, &["remote", "add", "origin", url])?
        .assert()
        .failure();
    command(&work_dir, &["remote", "show"])?.assert().success();
    command(&work_dir, &["remote", "rm", "origin"])?
        .assert()
        .success();
    command(&work_dir, &["remote", "rm", "origin"])?
        .assert()
        .failure();
    Ok(())
}
