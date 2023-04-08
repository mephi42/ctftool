use cookie_store::CookieStore;
use futures::future::FutureExt;
use regex::Regex;
use serde::Deserialize;
use url::Url;

use anyhow::{bail, Result};

use crate::ctf;
use crate::ctf::Remote;
use crate::engines;
use crate::http::{self, CookieStoreExt, RequestBuilderExt};

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

async fn get_login_page(
    client: &http::Client,
    cookie_store: &mut CookieStore,
    login_page_url: &Url,
) -> Result<String> {
    let login_page_request = client
        .get(login_page_url.to_string())
        .add_cookie_header(login_page_url, cookie_store);
    let login_page_response = client.execute(login_page_request.build()?).await?;
    login_page_response.error_for_status_ref()?;
    cookie_store.store_cookies_from_response(&login_page_response, login_page_url)?;
    let login_page = login_page_response.text().await?;
    let nonce_regex = Regex::new(r#"<input type="hidden" name="nonce" value="([^"]+)">"#)?;
    let nonces: Vec<_> = nonce_regex.captures_iter(&login_page).collect();
    let nonce = match nonces.as_slice() {
        [capture] => capture[1].to_owned(),
        _ => bail!("Could not find login nonce"),
    };
    Ok(nonce)
}

fn is_login_ok(response: &reqwest::Response) -> Result<bool> {
    if response.status() != 302 {
        return Ok(false);
    }
    let location = match response.headers().get(reqwest::header::LOCATION) {
        Some(location) => location.to_str()?,
        None => return Ok(false),
    };
    let location_url = Url::parse(location)?;
    Ok(location_url.path() == "/challenges")
}

async fn post_login_page(
    client: &http::Client,
    cookie_store: &mut CookieStore,
    login_page_url: &Url,
    username: String,
    password: String,
    nonce: String,
) -> Result<()> {
    let login_request = client
        .post(login_page_url.to_string())
        .multipart(
            reqwest::multipart::Form::new()
                .text("name", username)
                .text("password", password)
                .text("nonce", nonce),
        )
        .add_cookie_header(login_page_url, cookie_store);
    let login_response = client.execute(login_request.build()?).await?;
    login_response.error_for_status_ref()?;
    if !is_login_ok(&login_response)? {
        bail!("Incorrect login/password\n");
    }
    cookie_store.store_cookies_from_response(&login_response, login_page_url)?;
    Ok(())
}

async fn login(
    client: &http::Client,
    remote: &ctf::Remote,
    username: &str,
    password: &str,
) -> Result<CookieStore> {
    let mut cookie_store = CookieStore::default();
    let login_page_url = http::build_url(&remote.url, &["login"])?;
    let nonce = get_login_page(client, &mut cookie_store, &login_page_url).await?;
    post_login_page(
        client,
        &mut cookie_store,
        &login_page_url,
        username.to_string(),
        password.to_string(),
        nonce,
    )
    .await?;
    Ok(cookie_store)
}

async fn fetch_challenge(
    client: &http::Client,
    cookie_store: &CookieStore,
    remote: &ctf::Remote,
    challenge: Challenge,
) -> Result<ctf::Challenge> {
    let challenge_url = http::build_url(
        &remote.url,
        &["api", "v1", "challenges", &challenge.id.to_string()],
    )?;
    let challenge_request = client
        .get(challenge_url.as_str())
        .add_cookie_header(&challenge_url, cookie_store);
    let challenge_response = client.execute(challenge_request.build()?).await?;
    challenge_response.error_for_status_ref()?;
    let challenge_details: ChallengeDetails = challenge_response.json().await?;
    if !challenge_details.success {
        bail!("Could not retrieve challenge {}", challenge.id);
    }
    let category = ctf::best_category(&[challenge.category]);
    let title = ctf::sanitize_title(&challenge.name);
    let binaries =
        ctf::binaries_from_description(client, cookie_store, &challenge_details.data.description)
            .await?;
    let services = ctf::services_from_description(&challenge_details.data.description)?;
    Ok(ctf::Challenge {
        name: format!("{}-{}", category, title),
        description: challenge_details.data.description,
        binaries,
        services,
    })
}

pub async fn fetch(
    client: &http::Client,
    cookie_store: &CookieStore,
    remote: &ctf::Remote,
) -> Result<ctf::CTF> {
    let mut ctf = ctf::CTF::default();
    let challenges_url = http::build_url(&remote.url, &["api", "v1", "challenges"])?;
    let challenges_request = client
        .get(challenges_url.as_str())
        .add_cookie_header(&challenges_url, cookie_store);
    let challenges_response = client.execute(challenges_request.build()?).await?;
    challenges_response.error_for_status_ref()?;
    let challenges: Challenges = challenges_response.json().await?;
    if !challenges.success {
        bail!("Could not retrieve challenges");
    }
    for challenge in challenges.data {
        ctf.challenges
            .push(fetch_challenge(client, cookie_store, remote, challenge).await?);
    }
    Ok(ctf)
}

pub struct CtfdEngine {}

impl engines::Engine for CtfdEngine {
    fn detect<'a>(
        &self,
        _client: &'a http::Client,
        _remote: &'a ctf::Remote,
        main_page: &'a str,
    ) -> engines::DetectResult<'a> {
        engines::detect_needle(main_page, "Powered by CTFd").boxed()
    }

    fn login<'a>(
        &self,
        client: &'a http::Client,
        remote: &'a Remote,
        username: &'a str,
        password: &'a str,
    ) -> engines::LoginResult<'a> {
        login(client, remote, username, password).boxed()
    }

    fn fetch<'a>(
        &self,
        client: &'a http::Client,
        cookie_store: &'a CookieStore,
        remote: &'a ctf::Remote,
    ) -> engines::FetchResult<'a> {
        fetch(client, cookie_store, remote).boxed()
    }
}
