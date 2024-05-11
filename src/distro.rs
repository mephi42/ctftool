use anyhow::{anyhow, Result};
use elf::abi::{
    EM_386, EM_AARCH64, EM_ALPHA, EM_ARM, EM_AVR32, EM_IA_64, EM_MIPS, EM_PARISC, EM_PPC, EM_PPC64,
    EM_RISCV, EM_S390, EM_SH, EM_SPARC, EM_X86_64,
};
use elf::endian::{AnyEndian, EndianParse};
use elf::file::{Class, FileHeader};
use elf::ElfBytes;
use lazy_static::lazy_static;
use regex::bytes::Regex;
use std::default::Default;
use std::fs;
use std::path::PathBuf;
use std::str;

/// Distro and package versions.
#[derive(Default)]
pub struct Packages {
    pub arch: Option<&'static str>,
    pub distro: Option<&'static str>,
    pub distro_version: Option<&'static str>,
    pub libc_version: Option<String>,
}

/// Try to match a regex that has exactly one capture group.
fn try_regex_1(result: &mut Option<String>, regex: &Regex, bytes: &[u8]) -> Result<()> {
    if result.is_some() {
        return Ok(());
    }
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
    ehdr: Option<FileHeader<AnyEndian>>,
    debian_gcc_version: Option<String>,
    debian_libc_version: Option<String>,
    ubuntu_gcc_version: Option<String>,
    ubuntu_libc_version: Option<String>,
}

lazy_static! {
    static ref DEBIAN_GCC_REGEX: Regex = Regex::new(r"GCC: \(Debian (.+?)\)").unwrap();
    static ref DEBIAN_LIBC_REGEX: Regex =
        Regex::new(r"GNU C Library \(Debian GLIBC (.+?)\)").unwrap();
    static ref DEBIAN_LDSO_REGEX: Regex = Regex::new(r"ld\.so \(Debian GLIBC (.+?)\)").unwrap();
    static ref UBUNTU_GCC_REGEX: Regex = Regex::new(r"GCC: \(Ubuntu (.+?)\)").unwrap();
    static ref UBUNTU_LIBC_REGEX: Regex =
        Regex::new(r"GNU C Library \(Ubuntu GLIBC (.+?)\)").unwrap();
    static ref UBUNTU_LDSO_REGEX: Regex = Regex::new(r"ld\.so \(Ubuntu GLIBC (.+?)\)").unwrap();
}

impl BinaryInfo {
    fn analyze(path: &PathBuf) -> Result<BinaryInfo> {
        let mut result = BinaryInfo::default();
        let bytes = fs::read(path)?;
        if let Ok(elf) = ElfBytes::<AnyEndian>::minimal_parse(&bytes) {
            result.ehdr = Some(elf.ehdr);
        }
        try_regex_1(&mut result.debian_gcc_version, &DEBIAN_GCC_REGEX, &bytes)?;
        try_regex_1(&mut result.debian_libc_version, &DEBIAN_LIBC_REGEX, &bytes)?;
        try_regex_1(&mut result.debian_libc_version, &DEBIAN_LDSO_REGEX, &bytes)?;
        try_regex_1(&mut result.ubuntu_gcc_version, &UBUNTU_GCC_REGEX, &bytes)?;
        try_regex_1(&mut result.ubuntu_libc_version, &UBUNTU_LIBC_REGEX, &bytes)?;
        try_regex_1(&mut result.ubuntu_libc_version, &UBUNTU_LDSO_REGEX, &bytes)?;
        Ok(result)
    }
}

fn get_debian_arch_str(ehdr: &FileHeader<AnyEndian>) -> Option<&'static str> {
    if ehdr.e_machine == EM_386 {
        Some("i386")
    } else if ehdr.e_machine == EM_AARCH64 {
        Some("arm64")
    } else if ehdr.e_machine == EM_ALPHA {
        Some("alpha")
    } else if ehdr.e_machine == EM_ARM {
        Some("armhf")
    } else if ehdr.e_machine == EM_AVR32 {
        Some("avr32")
    } else if ehdr.e_machine == EM_IA_64 {
        Some("ia64")
    } else if ehdr.e_machine == EM_MIPS {
        if ehdr.class == Class::ELF32 {
            if ehdr.endianness.is_big() {
                Some("mips")
            } else {
                Some("mipsel")
            }
        } else if ehdr.class == Class::ELF64 {
            Some("mips64el")
        } else {
            None
        }
    } else if ehdr.e_machine == EM_PARISC {
        Some("hppa")
    } else if ehdr.e_machine == EM_PPC {
        Some("powerpc")
    } else if ehdr.e_machine == EM_PPC64 {
        if ehdr.endianness.is_big() {
            Some("ppc64")
        } else {
            Some("ppc64el")
        }
    } else if ehdr.e_machine == EM_RISCV {
        Some("riscv64")
    } else if ehdr.e_machine == EM_S390 {
        if ehdr.class == Class::ELF32 {
            Some("s390")
        } else if ehdr.class == Class::ELF64 {
            Some("s390x")
        } else {
            None
        }
    } else if ehdr.e_machine == EM_SH {
        Some("sh4")
    } else if ehdr.e_machine == EM_SPARC {
        if ehdr.class == Class::ELF32 {
            Some("sparc")
        } else if ehdr.class == Class::ELF64 {
            Some("sparc64")
        } else {
            None
        }
    } else if ehdr.e_machine == EM_X86_64 {
        Some("amd64")
    } else {
        None
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

pub static DEFAULT_ARCH: &str = "amd64";
pub static DEFAULT_DISTRO: &str = "ubuntu";
pub static DEFAULT_DISTRO_VERSION: &str = "latest";
pub static DEFAULT_LIBC_VERSION: &str = "*";

pub fn get_packages(path: &PathBuf) -> Result<Option<Packages>> {
    let info = BinaryInfo::analyze(path)?;
    let arch = info
        .ehdr
        .and_then(|ehdr| get_debian_arch_str(&ehdr))
        .unwrap_or(DEFAULT_ARCH);
    if let Some(debian_version) = get_debian_version(&info) {
        return Ok(Some(Packages {
            arch: Some(arch),
            distro: Some("debian"),
            distro_version: Some(debian_version),
            libc_version: info.debian_libc_version,
        }));
    }
    if let Some(ubuntu_version) = get_ubuntu_version(&info) {
        return Ok(Some(Packages {
            arch: Some(arch),
            distro: Some("ubuntu"),
            distro_version: Some(ubuntu_version),
            libc_version: info.ubuntu_libc_version,
        }));
    }
    Ok(None)
}

pub fn merge_packages_variants(packages_variants: Vec<Packages>) -> Packages {
    let mut packages = Packages::default();
    for packages_variant in packages_variants {
        packages.arch = packages.arch.or(packages_variant.arch);
        packages.distro = packages.distro.or(packages_variant.distro);
        packages.distro_version = packages.distro_version.or(packages_variant.distro_version);
        packages.libc_version = packages.libc_version.or(packages_variant.libc_version);
    }
    packages
}
