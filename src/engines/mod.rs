use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;

use crate::ctf;

pub mod ctfd;
pub mod watevr;

type FetchResult<'a> = Pin<Box<dyn Future<Output = Result<ctf::CTF>> + 'a>>;

pub trait Engine {
    fn fetch<'a>(&self, remote: &'a ctf::Remote) -> FetchResult<'a>;
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

pub async fn fetch_auto(remote: &ctf::Remote) -> Result<(String, ctf::CTF)> {
    let mut result: Result<(String, ctf::CTF)> =
        Err(anyhow!("Could not detect engine used by {}", remote.name));
    let mut errors = vec![];
    for (name, engine) in ENGINES.iter() {
        match engine.fetch(&remote).await {
            Ok(fetched) => match &result {
                Ok((_, best_fetched)) => {
                    if fetched.challenges.len() > best_fetched.challenges.len() {
                        result = Ok(((*name).to_owned(), fetched));
                    }
                }
                Err(_) => result = Ok(((*name).to_owned(), fetched)),
            },
            Err(e) => errors.push((name, e)),
        }
    }
    if result.is_err() {
        for (name, e) in errors {
            eprintln!("{}: {}", name, e);
        }
    }
    result
}
