use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::bytes::Regex;
use std::default::Default;
use std::fs;
use std::path::PathBuf;
use std::str;

/// Distro and package versions.
pub struct Packages {
    pub distro: String,
    pub distro_version: String,
    pub libc_version: String,
}

/// Try to match a regex that has exactly one capture group.
fn try_regex_1(result: &mut Option<String>, regex: &Regex, bytes: &[u8]) -> Result<()> {
    if let Some(m) = regex.captures_iter(bytes).next() {
        *result = Some(
            str::from_utf8(
                m.get(1)
                    .ok_or_else(|| anyhow!("Capture #1 missing"))?
                    .as_bytes(),
            )?
            .to_string(),
        )
    }
    Ok(())
}

/// Various information extracted from a binary.
#[derive(Default)]
struct BinaryInfo {
    debian_gcc_version: Option<String>,
    debian_libc_version: Option<String>,
    ubuntu_gcc_version: Option<String>,
    ubuntu_libc_version: Option<String>,
}

lazy_static! {
    static ref DEBIAN_GCC_REGEX: Regex = Regex::new(r"GCC: \(Debian (.+?)\)").unwrap();
    static ref DEBIAN_LIBC_REGEX: Regex =
        Regex::new(r"GNU C Library \(Debian GLIBC (.+?)\)").unwrap();
    static ref UBUNTU_GCC_REGEX: Regex = Regex::new(r"GCC: \(Ubuntu (.+?)\)").unwrap();
    static ref UBUNTU_LIBC_REGEX: Regex =
        Regex::new(r"GNU C Library \(Ubuntu GLIBC (.+?)\)").unwrap();
}

impl BinaryInfo {
    fn analyze(path: &PathBuf) -> Result<BinaryInfo> {
        let mut result = BinaryInfo::default();
        let bytes = fs::read(path)?;
        try_regex_1(&mut result.debian_gcc_version, &DEBIAN_GCC_REGEX, &bytes)?;
        try_regex_1(&mut result.debian_libc_version, &DEBIAN_LIBC_REGEX, &bytes)?;
        try_regex_1(&mut result.ubuntu_gcc_version, &UBUNTU_GCC_REGEX, &bytes)?;
        try_regex_1(&mut result.ubuntu_libc_version, &UBUNTU_LIBC_REGEX, &bytes)?;
        Ok(result)
    }
}

fn get_debian_versions_by_gcc_version(gcc_version: &str) -> &'static [&'static str] {
    if gcc_version.starts_with("6.") {
        &["9"]
    } else if gcc_version.starts_with("7.") || gcc_version.starts_with("8.") {
        &["10"]
    } else if gcc_version.starts_with("9.") || gcc_version.starts_with("10.") {
        &["11"]
    } else if gcc_version.starts_with("11.") || gcc_version.starts_with("12.") {
        &["12"]
    } else {
        &[]
    }
}

fn get_ubuntu_versions_by_gcc_version(gcc_version: &str) -> &'static [&'static str] {
    if gcc_version.starts_with("7.") {
        &["18.04"]
    } else if gcc_version.starts_with("9.") {
        &["20.04"]
    } else if gcc_version.starts_with("11.") {
        &["22.04"]
    } else if gcc_version.starts_with("12.") {
        &["22.10", "23.04"]
    } else {
        &[]
    }
}

fn get_debian_versions_by_libc_version(libc_version: &str) -> &'static [&'static str] {
    if libc_version.starts_with("2.24-") {
        &["9"]
    } else if libc_version.starts_with("2.28-") {
        &["10"]
    } else if libc_version.starts_with("2.31-") {
        &["11"]
    } else if libc_version.starts_with("2.36-") {
        &["12"]
    } else {
        &[]
    }
}

fn get_ubuntu_versions_by_libc_version(libc_version: &str) -> &'static [&'static str] {
    if libc_version.starts_with("2.27-") {
        &["18.04"]
    } else if libc_version.starts_with("2.31-") {
        &["20.04"]
    } else if libc_version.starts_with("2.35-") {
        &["22.04"]
    } else if libc_version.starts_with("2.36-") {
        &["22.10"]
    } else if libc_version.starts_with("2.37-") {
        &["23.04"]
    } else {
        &[]
    }
}

/// Returns an intersection of multiple slices.
fn get_intersection<'a>(slices: &[&[&'a str]]) -> Vec<&'a str> {
    if let Some(first) = slices.first() {
        first
            .iter()
            .copied()
            .filter(|x| {
                let rest = &slices[1..];
                rest.iter().all(|y| y.contains(x))
            })
            .collect()
    } else {
        Vec::new()
    }
}

fn get_distro_version<'a>(candidates: &[Option<&[&'a str]>]) -> Option<&'a str> {
    let known: Vec<&[&'a str]> = candidates.iter().filter_map(|x| *x).collect();
    get_intersection(&known).first().copied()
}

fn get_debian_version(info: &BinaryInfo) -> Option<&'static str> {
    get_distro_version(&[
        info.debian_gcc_version
            .as_ref()
            .map(|x| get_debian_versions_by_gcc_version(x)),
        info.debian_libc_version
            .as_ref()
            .map(|x| get_debian_versions_by_libc_version(x)),
    ])
}

fn get_ubuntu_version(info: &BinaryInfo) -> Option<&'static str> {
    get_distro_version(&[
        info.ubuntu_gcc_version
            .as_ref()
            .map(|x| get_ubuntu_versions_by_gcc_version(x)),
        info.ubuntu_libc_version
            .as_ref()
            .map(|x| get_ubuntu_versions_by_libc_version(x)),
    ])
}

impl Default for Packages {
    fn default() -> Packages {
        Packages {
            distro: "ubuntu".into(),
            distro_version: "latest".into(),
            libc_version: "*".into(),
        }
    }
}

pub fn get_packages(path: &PathBuf) -> Result<Option<Packages>> {
    let info = BinaryInfo::analyze(path)?;
    if let Some(debian_version) = get_debian_version(&info) {
        return Ok(Some(Packages {
            distro: "debian".into(),
            distro_version: debian_version.into(),
            libc_version: info.debian_libc_version.unwrap_or_else(|| "*".into()),
        }));
    }
    if let Some(ubuntu_version) = get_ubuntu_version(&info) {
        return Ok(Some(Packages {
            distro: "ubuntu".into(),
            distro_version: ubuntu_version.into(),
            libc_version: info.ubuntu_libc_version.unwrap_or_else(|| "*".into()),
        }));
    }
    Ok(None)
}
