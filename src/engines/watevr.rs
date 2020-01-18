use cookie_store::CookieStore;
use futures::future::{self, FutureExt};
use serde::Deserialize;

use anyhow::Result;

use crate::ctf;
use crate::engines;
use crate::http::{self, RequestBuilderExt};

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

async fn fetch(
    client: &http::Client,
    cookie_store: &CookieStore,
    remote: &ctf::Remote,
) -> Result<ctf::CTF> {
    let mut ctf = ctf::CTF::default();
    let url = http::build_url(&remote.url, &["api", "watsup"])?;
    let request = client
        .get(&url.to_string())
        .add_cookie_header(&url, &cookie_store);
    let response = client.execute(request.build()?).await?;
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

pub struct WatevrEngine {}

impl engines::Engine for WatevrEngine {
    fn detect<'a>(
        &self,
        _client: &'a http::Client,
        _remote: &'a ctf::Remote,
        main_page: &'a str,
    ) -> engines::DetectResult<'a> {
        engines::detect_needle(main_page, "watevrCTF").boxed()
    }

    fn login<'a>(
        &self,
        _client: &'a http::Client,
        _remote: &'a ctf::Remote,
        _login: &'a str,
        _password: &'a str,
    ) -> engines::LoginResult<'a> {
        future::ok(CookieStore::default()).boxed()
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
