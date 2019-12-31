use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_yaml;
use url::Url;

use anyhow::{anyhow, Error, Result};
use lazy_static::lazy_static;

#[derive(Serialize, Deserialize)]
pub struct CTF {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub remotes: Vec<Remote>,
    #[serde(default)]
    pub challenges: Vec<Challenge>,
}

#[derive(Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Challenge {
    pub name: String,
    pub description: String,
    pub binaries: Vec<Binary>,
    pub services: Vec<Service>,
}

#[derive(Serialize, Deserialize)]
pub struct Binary {
    pub name: String,
    pub alternatives: Vec<BinaryAlternative>,
}

#[derive(Serialize, Deserialize)]
pub struct Checksum {
    pub algorithm: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct BinaryAlternative {
    pub name: String,
    pub url: Option<String>,
    #[serde(default)]
    pub checksum: Option<Checksum>,
}

#[derive(Serialize, Deserialize)]
pub struct Service {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

pub struct Context {
    pub ctf: CTF,
    pub root: PathBuf,
    pub path: Vec<String>,
}

pub fn load() -> Result<Context> {
    let mut path = Vec::new();
    let mut dir = env::current_dir()?;
    loop {
        match fs::read(dir.join(".ctf")) {
            Ok(bytes) => {
                let str = &String::from_utf8(bytes)?;
                let ctf: CTF = serde_yaml::from_str(str)?;
                path.reverse();
                break Ok(Context {
                    ctf,
                    root: dir,
                    path,
                });
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                if !dir.pop() {
                    break Err(anyhow!(
                        "No .ctf file in the current or any of the parent directories"
                    ));
                }
            }
            Err(e) => break Err(Error::new(e)),
        }
    }
}

fn ignore(ctf: &CTF) -> Vec<String> {
    let mut result = vec!["/*".into(), "!/.ctf".into(), "!/.gitignore".into()];
    for challenge in &ctf.challenges {
        result.push(format!("!/{}/", challenge.name));
        result.push(format!("/{}/*", challenge.name));
        for binary in &challenge.binaries {
            for alternative in &binary.alternatives {
                result.push(format!(
                    "!/{}/{}.{}",
                    challenge.name, binary.name, alternative.name
                ));
            }
        }
    }
    result
}

pub fn store(ctf: &CTF) -> Result<()> {
    fs::write(".gitignore", ignore(&ctf).join("\n"))?;
    fs::write(".ctf", serde_yaml::to_string(&ctf)?)?;
    Ok(())
}

lazy_static! {
    static ref CATEGORY_PRIORITIES: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert("crypto", 0);
        m.insert("web", 1);
        m.insert("forensics", 2);
        m.insert("pwn", 3);
        m.insert("reverse", 4);
        m
    };
}

pub fn best_category(categories: &[String]) -> &str {
    categories
        .iter()
        .map(|category| category.as_str())
        .min_by_key(|category| CATEGORY_PRIORITIES.get(category).unwrap_or(&999))
        .unwrap_or(&"misc")
}

pub fn sanitize_title(title: &str) -> String {
    title.replace(" ", "_")
}

pub fn binary_from_url(url: &str) -> Result<Binary> {
    let parsed = Url::parse(url)?;
    let path_segments = parsed
        .path_segments()
        .ok_or_else(|| anyhow!("cannot be base"))?;
    let name = path_segments
        .last()
        .ok_or_else(|| anyhow!("cannot be empty"))?;
    Ok(Binary {
        name: name.into(),
        alternatives: vec![BinaryAlternative {
            name: "orig".into(),
            url: Some(url.into()),
            checksum: None,
        }],
    })
}

fn merge_binary_alternatives(
    binary_alternative: &mut BinaryAlternative,
    binary_alternative2: BinaryAlternative,
) {
    if binary_alternative2.url.is_some() {
        binary_alternative.url = binary_alternative2.url;
    }
    if binary_alternative2.checksum.is_some() {
        binary_alternative.checksum = binary_alternative2.checksum;
    }
}

fn merge_binaries(binary: &mut Binary, binary2: Binary) {
    for binary_alternative2 in binary2.alternatives {
        let existing = binary
            .alternatives
            .iter_mut()
            .find(|binary_alternative| binary_alternative.name == binary_alternative2.name);
        match existing {
            Some(binary_alternative) => {
                merge_binary_alternatives(binary_alternative, binary_alternative2)
            }
            None => binary.alternatives.push(binary_alternative2),
        }
    }
}

fn merge_services(service: &mut Service, service2: Service) {
    if service2.name.is_some() {
        service.name = service2.name;
    }
    if service2.url.is_some() {
        service.url = service2.url;
    }
}

fn merge_challenges(challenge: &mut Challenge, challenge2: Challenge) {
    if !challenge2.description.is_empty() {
        challenge.description = challenge2.description;
    }
    for binary2 in challenge2.binaries {
        let existing = challenge
            .binaries
            .iter_mut()
            .find(|binary| binary.name == binary2.name);
        match existing {
            Some(binary) => merge_binaries(binary, binary2),
            None => challenge.binaries.push(binary2),
        }
    }
    for service2 in challenge2.services {
        let existing = challenge.services.iter_mut().find(|service| {
            service.name.is_some() && service.name == service2.name
                || service.url.is_some() && service.url == service2.url
        });
        match existing {
            Some(service) => merge_services(service, service2),
            None => challenge.services.push(service2),
        }
    }
}

pub fn merge(ctf: &mut CTF, ctf2: CTF) {
    if !ctf2.name.is_empty() {
        ctf.name = ctf2.name;
    }
    for challenge2 in ctf2.challenges {
        let existing = ctf
            .challenges
            .iter_mut()
            .find(|challenge| challenge.name == challenge2.name);
        match existing {
            Some(challenge) => merge_challenges(challenge, challenge2),
            None => ctf.challenges.push(challenge2),
        }
    }
}

pub fn find_challenge<'a>(ctf: &'a CTF, name: &str) -> Result<&'a Challenge> {
    ctf.challenges
        .iter()
        .find(|challenge| challenge.name == name)
        .ok_or_else(|| anyhow!("No such challenge: {}", name))
}

pub fn find_challenge_mut<'a>(ctf: &'a mut CTF, name: &str) -> Result<&'a mut Challenge> {
    ctf.challenges
        .iter_mut()
        .find(|challenge| challenge.name == name)
        .ok_or_else(|| anyhow!("No such challenge: {}", name))
}

pub fn find_binary<'a>(challenge: &'a Challenge, name: &str) -> Result<&'a Binary> {
    challenge
        .binaries
        .iter()
        .find(|binary| binary.name == name)
        .ok_or_else(|| anyhow!("No such binary: {}", name))
}

pub fn find_binary_mut<'a>(challenge: &'a mut Challenge, name: &str) -> Result<&'a mut Binary> {
    challenge
        .binaries
        .iter_mut()
        .find(|binary| binary.name == name)
        .ok_or_else(|| anyhow!("No such binary: {}", name))
}

pub fn find_alternative<'a>(binary: &'a Binary, name: &str) -> Result<&'a BinaryAlternative> {
    binary
        .alternatives
        .iter()
        .find(|alternative| alternative.name == name)
        .ok_or_else(|| anyhow!("No such alternative: {}", name))
}

pub fn find_alternative_mut<'a>(
    binary: &'a mut Binary,
    name: &str,
) -> Result<&'a mut BinaryAlternative> {
    binary
        .alternatives
        .iter_mut()
        .find(|alternative| alternative.name == name)
        .ok_or_else(|| anyhow!("No such alternative: {}", name))
}
