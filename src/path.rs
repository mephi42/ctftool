use crate::os_str::os_str_to_str;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub fn relativize(root: &Path, cwd: &Path, path: PathBuf) -> Result<(PathBuf, PathBuf)> {
    let canonical_root = root.canonicalize()?;
    let canonical_path = if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
    .canonicalize()?;
    let relative_path = canonical_path
        .strip_prefix(canonical_root)
        .map_err(|_| anyhow!("{} is not in the CTF directory", &canonical_path.display()))?
        .to_path_buf();
    Ok((canonical_path, relative_path))
}

pub fn path_to_str(path: &Path) -> Result<&str> {
    os_str_to_str(path.as_os_str())
}
