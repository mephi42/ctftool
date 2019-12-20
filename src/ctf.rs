use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_yaml;
use url::Url;

use anyhow::{anyhow, Result};
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
pub struct BinaryAlternative {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Service {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

pub fn load() -> Result<CTF> {
    let bytes = fs::read(".ctf")?;
    let str = &String::from_utf8(bytes)?;
    let ctf = serde_yaml::from_str(str)?;
    Ok(ctf)
}

pub fn store(ctf: &CTF) -> Result<()> {
    fs::write(
        ".gitignore",
        "/*
!/.ctf
!/.gitignore
",
    )?;
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
