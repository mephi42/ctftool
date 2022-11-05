use crate::http;
use anyhow::{anyhow, Result};
use regex::bytes::{Captures, Regex};
use std::fs;
use std::path::PathBuf;
use std::str;

pub struct Packages {
    pub distro: String,
    pub distro_version: String,
    pub libc_version: String,
}

fn decode_capture_1(m: Captures) -> Result<String> {
    Ok(str::from_utf8(
        m.get(1)
            .ok_or_else(|| anyhow!("Capture #1 missing"))?
            .as_bytes(),
    )?
    .to_string())
}

fn get_ubuntu_libc_version(path: &PathBuf) -> Result<Option<String>> {
    let bytes = fs::read(path)?;
    let version_regex = Regex::new(r"GNU C Library \(Ubuntu GLIBC (.+?)\)")?;
    if let Some(m) = version_regex.captures_iter(&bytes).next() {
        return Ok(Some(decode_capture_1(m)?));
    }
    Ok(None)
}

async fn get_ubuntu_version_by_libc_version(
    client: &http::Client,
    libc_version: &str,
) -> Result<Option<String>> {
    let changelog_url = http::build_url(
        "http://changelogs.ubuntu.com",
        &[
            "changelogs",
            "pool",
            "main",
            "g",
            "glibc",
            &format!("glibc_{}", libc_version),
            "changelog",
        ],
    )?;
    let changelog_request = client.get(changelog_url.as_str());
    let changelog_response = client.execute(changelog_request.build()?).await?;
    changelog_response.error_for_status_ref()?;
    let changelog = changelog_response.bytes().await?;
    let version_regex = Regex::new(r"glibc \((?:.+?)\) (.+?);")?;
    if let Some(m) = version_regex.captures_iter(&changelog).next() {
        return Ok(Some(decode_capture_1(m)?));
    }
    Ok(None)
}

impl Packages {
    pub fn default() -> Packages {
        Packages {
            distro: "ubuntu".into(),
            distro_version: "latest".into(),
            libc_version: "*".into(),
        }
    }
}

pub async fn get_packages(client: &http::Client, path: &PathBuf) -> Result<Option<Packages>> {
    if let Some(libc_version) = get_ubuntu_libc_version(path)? {
        if let Some(distro_version) =
            get_ubuntu_version_by_libc_version(client, &libc_version).await?
        {
            return Ok(Some(Packages {
                distro: "ubuntu".into(),
                distro_version,
                libc_version,
            }));
        }
    }
    Ok(None)
}
