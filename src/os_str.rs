use anyhow::{anyhow, Result};
use std::ffi::OsStr;

pub fn os_str_to_str(os_str: &OsStr) -> Result<&str> {
    os_str
        .to_str()
        .ok_or_else(|| anyhow!("{:?} contains non-UTF-8 characters", os_str))
}
