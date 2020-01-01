use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml;
use url::Url;

use anyhow::{anyhow, bail, Error, Result};
use lazy_static::lazy_static;

#[derive(Default, Serialize, Deserialize)]
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
    #[serde(default = "default_engine")]
    pub engine: String,
}

pub fn default_engine() -> String {
    "auto".into()
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

#[derive(Default, Serialize, Deserialize)]
pub struct Credentials {
    pub remotes: Vec<RemoteCredentials>,
}

#[derive(Serialize, Deserialize)]
pub struct RemoteCredentials {
    pub name: String,
    pub cookies: String,
}

pub struct Context {
    pub ctf: CTF,
    pub credentials: Credentials,
    pub root: PathBuf,
    pub path: Vec<String>,
}

fn load_credentials(root: &Path) -> Result<Credentials> {
    match fs::read(root.join(".ctfcredentials")) {
        Ok(bytes) => {
            let str = &String::from_utf8(bytes)?;
            Ok(serde_yaml::from_str(str)?)
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Credentials::default()),
        Err(e) => Err(Error::new(e)),
    }
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
                    credentials: load_credentials(&dir)?,
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

pub fn store(context: &Context) -> Result<()> {
    fs::write(
        context.root.join(".gitignore"),
        ignore(&context.ctf).join("\n"),
    )?;
    fs::write(
        context.root.join(".ctf"),
        serde_yaml::to_string(&context.ctf)?,
    )?;
    fs::write(
        context.root.join(".ctfcredentials"),
        serde_yaml::to_string(&context.credentials)?,
    )?;
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

pub fn best_category(categories: &[String]) -> String {
    categories
        .iter()
        .map(|category| category.to_lowercase())
        .min_by_key(|category| CATEGORY_PRIORITIES.get(category.as_str()).unwrap_or(&999))
        .unwrap_or_else(|| "misc".to_owned())
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

fn url_regex() -> Result<Regex> {
    let result = Regex::new(r#"https?://[^\s"]+"#)?;
    Ok(result)
}

fn extract_google_drive_id(url: &str) -> Result<Option<String>> {
    let regexes = vec![
        Regex::new(r#"^https://drive.google.com/file/d/(.+)/view$"#)?,
        Regex::new(r#"^https://drive.google.com/open\?id=(.+)$"#)?,
    ];
    for regex in regexes {
        let id = regex.captures(&url).map(|cap| cap[1].to_string());
        if id.is_some() {
            return Ok(id);
        }
    }
    Ok(None)
}

async fn resolve_google_drive_binary(client: &reqwest::Client, id: &str) -> Result<Binary> {
    let initial_url = format!("https://drive.google.com/uc?export=download&id={}", id);
    let mut url = initial_url.clone();
    loop {
        let response = client.execute(client.get(&url).build()?).await?;
        if response.status() == 302 {
            url = match response.headers().get(reqwest::header::LOCATION) {
                Some(location) => location.to_str()?.to_string(),
                None => bail!("302, but no Location"),
            };
            continue;
        }
        response.error_for_status_ref()?;
        let content_disposition = match response.headers().get(reqwest::header::CONTENT_DISPOSITION)
        {
            Some(location) => location.to_str()?,
            None => bail!("No Content-Disposition"),
        };
        let content_disposition_regex = Regex::new(r#"^attachment;filename="([^"]+)";"#)?;
        if let Some(file_name) = content_disposition_regex
            .captures(&content_disposition)
            .map(|cap| cap[1].to_string())
        {
            break Ok(Binary {
                name: file_name,
                alternatives: vec![BinaryAlternative {
                    name: "orig".into(),
                    url: Some(initial_url),
                    checksum: None,
                }],
            });
        }
        bail!("No attachment in Content-Disposition");
    }
}

pub fn services_from_description(description: &str) -> Result<Vec<Service>> {
    let mut services = Vec::new();
    let nc_regex = Regex::new(r#"nc ([^ ]+) (\d+)"#)?;
    for nc in nc_regex.captures_iter(&description) {
        services.push(Service {
            name: None,
            url: Some(format!("nc://{}:{}", &nc[1], &nc[2])),
        });
    }
    Ok(services)
}

pub async fn binaries_from_description(
    client: &reqwest::Client,
    description: &str,
) -> Result<Vec<Binary>> {
    let mut binaries = Vec::new();
    let urls: Vec<String> = url_regex()?
        .captures_iter(&description)
        .map(|cap| cap[0].to_string())
        .collect();
    for url in urls {
        if let Some(id) = extract_google_drive_id(&url)? {
            binaries.push(resolve_google_drive_binary(client, &id).await?);
        }
    }
    Ok(binaries)
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

pub fn find_remote<'a>(ctf: &'a CTF, name: &str) -> Result<&'a Remote> {
    ctf.remotes
        .iter()
        .find(|remote| remote.name == name)
        .ok_or_else(|| anyhow!("Remote {} does not exist", name))
}

pub fn find_remote_mut<'a>(ctf: &'a mut CTF, name: &str) -> Result<&'a mut Remote> {
    ctf.remotes
        .iter_mut()
        .find(|remote| remote.name == name)
        .ok_or_else(|| anyhow!("Remote {} does not exist", name))
}

pub fn set_cookies(credentials: &mut Credentials, remote_name: String, cookies: String) {
    for remote in &mut credentials.remotes {
        if remote.name == remote_name {
            remote.cookies = cookies;
            return;
        }
    }
    credentials.remotes.push(RemoteCredentials {
        name: remote_name,
        cookies,
    });
}
