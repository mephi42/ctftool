use std::env;

use futures::future::FutureExt;
use regex::Regex;
use serde::Deserialize;
use url::Url;

use anyhow::{anyhow, bail, Result};

use crate::ctf;
use crate::engines;

#[derive(Deserialize)]
struct Challenges {
    success: bool,
    data: Vec<Challenge>,
}

#[derive(Deserialize)]
struct Challenge {
    id: i32,
    name: String,
    category: String,
}

#[derive(Deserialize)]
struct ChallengeDetails {
    success: bool,
    data: ChallengeDetailsData,
}

#[derive(Deserialize)]
struct ChallengeDetailsData {
    description: String,
}

async fn login(client: &reqwest::Client, login_page_url: &str) -> Result<()> {
    let login_page_response = client.execute(client.get(login_page_url).build()?).await?;
    login_page_response.error_for_status_ref()?;
    let login_page = login_page_response.text().await?;
    let nonce_regex = Regex::new(r#"<input type="hidden" name="nonce" value="([^"]+)">"#)?;
    let nonces: Vec<_> = nonce_regex.captures_iter(&login_page).collect();
    let nonce = match nonces.as_slice() {
        [capture] => capture[1].to_owned(),
        _ => bail!("Could not find login nonce"),
    };
    let login_response = client
        .execute(
            client
                .post(login_page_url)
                .multipart(
                    reqwest::multipart::Form::new()
                        // TODO: Environment variables are a temporary solution.
                        // TODO: They should be replaced by `ctf login` command.
                        .text("name", env::var("CTFTOOL_LOGIN")?)
                        .text("password", env::var("CTFTOOL_PASSWORD")?)
                        .text("nonce", nonce),
                )
                .build()?,
        )
        .await?;
    login_response.error_for_status_ref()?;
    Ok(())
}

async fn fetch_challenge(
    client: &reqwest::Client,
    url: &Url,
    challenge: Challenge,
) -> Result<ctf::Challenge> {
    let mut challenge_url = url.clone();
    challenge_url
        .path_segments_mut()
        .map_err(|_| anyhow!("cannot be base"))?
        .extend(&["api", "v1", "challenges", &challenge.id.to_string()]);
    let challenge_response = client
        .execute(client.get(challenge_url.as_str()).build()?)
        .await?;
    challenge_response.error_for_status_ref()?;
    let challenge_details: ChallengeDetails = challenge_response.json().await?;
    if !challenge_details.success {
        bail!("Could not retrieve challenge {}", challenge.id);
    }
    let category = ctf::best_category(&[challenge.category]);
    let title = ctf::sanitize_title(&challenge.name);
    let services = ctf::services_from_description(&challenge_details.data.description)?;
    Ok(ctf::Challenge {
        name: format!("{}-{}", category, title),
        description: challenge_details.data.description,
        binaries: vec![],
        services,
    })
}

pub async fn fetch(remote: &ctf::Remote) -> Result<ctf::CTF> {
    let mut ctf = ctf::CTF::default();
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let url = Url::parse(&remote.url)?;
    let mut challenges_url = url.clone();
    challenges_url
        .path_segments_mut()
        .map_err(|_| anyhow!("cannot be base"))?
        .extend(&["api", "v1", "challenges"]);
    for i in 0..=1 {
        let challenges_response = client
            .execute(client.get(challenges_url.as_str()).build()?)
            .await?;
        if i == 0 && challenges_response.status() == 302 {
            let mut login_page_url = url.clone();
            login_page_url
                .path_segments_mut()
                .map_err(|_| anyhow!("cannot be base"))?
                .extend(&["login"]);
            login(&client, login_page_url.as_str()).await?;
            continue;
        }
        challenges_response.error_for_status_ref()?;
        let challenges: Challenges = challenges_response.json().await?;
        if !challenges.success {
            bail!("Could not retrieve challenges");
        }
        for challenge in challenges.data {
            ctf.challenges
                .push(fetch_challenge(&client, &url, challenge).await?);
        }
    }
    Ok(ctf)
}

pub struct CtfdEngine {}

impl engines::Engine for CtfdEngine {
    fn fetch<'a>(&self, remote: &'a ctf::Remote) -> engines::FetchResult<'a> {
        fetch(remote).boxed()
    }
}
