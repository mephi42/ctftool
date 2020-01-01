use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use cookie_store::CookieStore;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;

use crate::ctf;

pub mod ctfd;
pub mod watevr;

type DetectResult<'a> = Pin<Box<dyn Future<Output = Result<()>> + 'a>>;
type LoginResult<'a> = Pin<Box<dyn Future<Output = Result<CookieStore>> + 'a>>;
type FetchResult<'a> = Pin<Box<dyn Future<Output = Result<ctf::CTF>> + 'a>>;

pub trait Engine {
    fn detect<'a>(
        &self,
        client: &'a reqwest::Client,
        remote: &'a ctf::Remote,
        main_page: &'a str,
    ) -> DetectResult<'a>;
    fn login<'a>(
        &self,
        client: &'a reqwest::Client,
        remote: &'a ctf::Remote,
        login: &'a str,
        password: &'a str,
    ) -> LoginResult<'a>;
    fn fetch<'a>(
        &self,
        client: &'a reqwest::Client,
        cookie_store: &'a CookieStore,
        remote: &'a ctf::Remote,
    ) -> FetchResult<'a>;
}

lazy_static! {
    static ref ENGINES: HashMap<&'static str, Box<dyn Engine + Sync>> = {
        let mut m = HashMap::new();
        m.insert(
            "ctfd",
            Box::new(ctfd::CtfdEngine {}) as Box<dyn Engine + Sync>,
        );
        m.insert(
            "watevr",
            Box::new(watevr::WatevrEngine {}) as Box<dyn Engine + Sync>,
        );
        m
    };
}

pub fn get_engine(name: &str) -> Result<&(dyn Engine + Sync)> {
    ENGINES
        .get(name)
        .map(|x| x.as_ref())
        .ok_or_else(|| anyhow!("Unsupported engine: {}", name))
}

pub async fn detect(client: &reqwest::Client, remote: &ctf::Remote) -> Result<String> {
    let main_page_response = client.execute(client.get(&remote.url).build()?).await?;
    main_page_response.error_for_status_ref()?;
    let main_page = main_page_response.text().await?;
    let mut errors = vec![];
    for (name, engine) in ENGINES.iter() {
        if let Err(e) = engine.detect(client, remote, &main_page).await {
            errors.push((name, e));
        } else {
            return Ok((*name).to_string());
        }
    }
    for (name, e) in errors {
        eprintln!("{}: {}", name, e);
    }
    Err(anyhow!("Could not detect engine used by {}", remote.name))
}
