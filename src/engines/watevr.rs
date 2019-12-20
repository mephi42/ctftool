use serde::Deserialize;
use url::Url;

use anyhow::{anyhow, Result};

use crate::ctf;

#[derive(Deserialize)]
struct Watsup {
    challenges: Vec<Challenge>,
}

#[derive(Deserialize)]
struct Challenge {
    categories: Vec<String>,
    authors: Vec<String>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    website_urls: Vec<String>,
    #[serde(default)]
    netcat_ips: Vec<String>,
    #[serde(default)]
    file_urls: Vec<String>,
    title: String,
}

pub async fn fetch(remote: &ctf::Remote) -> Result<ctf::CTF> {
    let mut ctf = ctf::CTF {
        name: "".into(),
        remotes: vec![],
        challenges: vec![],
    };
    let mut url = Url::parse(&remote.url)?;
    url.path_segments_mut()
        .map_err(|_| anyhow!("cannot be base"))?
        .extend(&["api", "watsup"]);
    let response = reqwest::get(&url.into_string()).await?;
    response.error_for_status_ref()?;
    let watsup: Watsup = response.json().await?;
    for challenge in watsup.challenges {
        let category = ctf::best_category(&challenge.categories);
        let title = ctf::sanitize_title(&challenge.title);
        ctf.challenges.push(ctf::Challenge {
            name: format!("{}-{}", category, title),
            description: format!(
                "Name: {}
Categories: {}
Authors: {}
Description: {}",
                challenge.title,
                challenge.categories.join(", "),
                challenge.authors.join(", "),
                challenge.description
            ),
            binaries: challenge
                .file_urls
                .iter()
                .filter_map(|url| match ctf::binary_from_url(url.as_str()) {
                    Ok(binary) => Some(binary),
                    Err(e) => {
                        println!("{}", e);
                        None
                    }
                })
                .collect(),
            services: challenge
                .website_urls
                .iter()
                .map(|url| ctf::Service {
                    name: None,
                    url: Some(url.into()),
                })
                .chain(challenge.netcat_ips.iter().map(|url| ctf::Service {
                    name: None,
                    url: Some(format!("nc://{}", url)),
                }))
                .collect(),
        })
    }
    Ok(ctf)
}
