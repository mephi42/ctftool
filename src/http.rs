use reqwest::header::HeaderValue;
use reqwest::{RequestBuilder, Url};

use anyhow::{anyhow, Result};

pub fn build_url(base: &str, path: &[&str]) -> Result<Url> {
    let mut url = Url::parse(base)?;
    url.path_segments_mut()
        .map_err(|_| anyhow!("cannot be base"))?
        .extend(path);
    Ok(url)
}

pub trait RequestBuilderExt {
    fn add_cookie_header(self, url: &Url, cookie_store: &cookie_store::CookieStore) -> Self;
}

impl RequestBuilderExt for RequestBuilder {
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

pub fn mk_client() -> Result<reqwest::Client> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    Ok(client)
}
