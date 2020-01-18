use reqwest::header::HeaderValue;
use reqwest::{self, Url};

use anyhow::{anyhow, Result};
use regex::Regex;

use crate::ctf;

pub fn build_url<I>(base: &str, path: I) -> Result<Url>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut url = Url::parse(base)?;
    url.path_segments_mut()
        .map_err(|_| anyhow!("cannot be base"))?
        .extend(path);
    Ok(url)
}

pub trait RequestBuilderExt {
    fn add_cookie_header(self, url: &Url, cookie_store: &cookie_store::CookieStore) -> Self;
}

impl RequestBuilderExt for reqwest::RequestBuilder {
    /* Stolen from reqwest::async_impl::client. */
    fn add_cookie_header(self, url: &Url, cookie_store: &cookie_store::CookieStore) -> Self {
        let header = cookie_store
            .get_request_cookies(url)
            .map(|c| format!("{}={}", c.name(), c.value()))
            .collect::<Vec<_>>()
            .join("; ");
        if header.is_empty() {
            self
        } else {
            self.header(
                reqwest::header::COOKIE,
                HeaderValue::from_bytes(header.as_bytes()).unwrap(),
            )
        }
    }
}

pub trait CookieStoreExt {
    fn store_cookies_from_response(
        &mut self,
        response: &reqwest::Response,
        url: &Url,
    ) -> Result<()>;
}

impl CookieStoreExt for cookie_store::CookieStore {
    /* Stolen from reqwest::async_impl::client and reqwest::cookie. */
    fn store_cookies_from_response(
        &mut self,
        response: &reqwest::Response,
        url: &Url,
    ) -> Result<()> {
        let mut cookies = vec![];
        for header_value in response
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
        {
            let mut cookie = cookie::Cookie::parse(header_value.to_str()?.to_owned())?;
            if cookie.max_age().is_none() && cookie.expires().is_none() {
                /* Nasty hack: make all cookies persistent, so that CookieStore.save_json() would
                output them. */
                cookie.set_max_age(time::Duration::weeks(1));
            }
            cookies.push(cookie);
        }
        self.store_response_cookies(cookies.into_iter(), &url);
        Ok(())
    }
}

struct RewriteRule {
    regex: Regex,
    rep: String,
}

pub struct Client {
    client: reqwest::Client,
    rewrite_rules: Vec<RewriteRule>,
}

impl Client {
    pub async fn execute(&self, mut request: reqwest::Request) -> Result<reqwest::Response> {
        self.rewrite(request.url_mut())?;
        Ok(self.client.execute(request).await?)
    }

    pub fn get<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.client.get(url)
    }

    pub fn post<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.client.post(url)
    }

    fn rewrite(&self, url: &mut Url) -> Result<()> {
        let mut s = url.as_str().to_owned();
        for rewrite_rule in &self.rewrite_rules {
            s = rewrite_rule
                .regex
                .replace_all(&s, rewrite_rule.rep.as_str())
                .into_owned();
        }
        *url = Url::parse(&s)?;
        Ok(())
    }
}

pub fn mk_client(rewrite_rule_strings: &[ctf::RewriteRule]) -> Result<Client> {
    let mut rewrite_rules = Vec::new();
    for s in rewrite_rule_strings {
        rewrite_rules.push(RewriteRule {
            regex: Regex::new(&s.regex)?,
            rep: s.rep.to_owned(),
        });
    }
    Ok(Client {
        client: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?,
        rewrite_rules,
    })
}
