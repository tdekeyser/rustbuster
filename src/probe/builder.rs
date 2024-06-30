use std::collections::HashMap;

use reqwest::{Client, Method, redirect};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use reqwest::Url;

use crate::{Error, Result};
use crate::probe::{FUZZ, HttpProbe};

pub struct HttpProbeBuilder {
    url: Url,
    method: Method,
    headers: HeaderMap,
    fuzzed_headers: HashMap<String, String>,
}

impl HttpProbeBuilder {
    pub fn new() -> HttpProbeBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rustbuster"));

        HttpProbeBuilder {
            url: "http://localhost:8080/FUZZ".parse().unwrap(),
            headers,
            method: Method::GET,
            fuzzed_headers: HashMap::new(),
        }
    }

    pub fn build(self) -> Result<HttpProbe> {
        self.validate()?;

        let client = Client::builder()
            .default_headers(self.headers)
            .redirect(redirect::Policy::none())
            .build()?;

        Ok(HttpProbe {
            url: self.url,
            client,
            method: self.method,
            fuzzed_headers: self.fuzzed_headers,
        })
    }

    fn validate(&self) -> Result<()> {
        match self.url.as_str().contains(FUZZ) || self.fuzzed_headers.iter().count() > 0 {
            true => Ok(()),
            false => Err(Error::FuzzKeywordNotFound)
        }
    }

    pub fn with_url(mut self, url: Url) -> HttpProbeBuilder {
        self.url = url;
        self
    }

    pub fn with_method(mut self, method: Method) -> HttpProbeBuilder {
        self.method = method;
        self
    }

    pub fn with_headers(mut self, headers: Vec<(HeaderName, HeaderValue)>) -> HttpProbeBuilder {
        self.headers.extend(headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<HeaderName, HeaderValue>>());

        self.fuzzed_headers.extend(
            headers.iter()
                .filter(|(k, v)| format!("{:?}{:?}", k, v).contains("FUZZ"))
                .map(|(k, v)| (k.clone().to_string(), String::from(v.clone().to_str().unwrap_or_default())))
                .collect::<HashMap<String, String>>());

        self
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use reqwest::header::{COOKIE, USER_AGENT};

    use crate::probe::HttpProbe;

    #[test]
    fn error_when_no_fuzz_keyword_found() -> Result<(), Box<dyn Error>> {
        match HttpProbe::builder()
            .with_url("http://localhost:9999".parse()?)
            .build() {
            Ok(_) => Err("not supposed to succeed".into()),
            Err(e) => {
                match e {
                    crate::Error::FuzzKeywordNotFound => Ok(()),
                    _ => Err("wrong error type".into())
                }
            }
        }
    }

    #[test]
    fn headers_containing_fuzz_are_fuzzed_headers() -> Result<(), Box<dyn Error>> {
        let builder = HttpProbe::builder()
            .with_headers(vec![
                (USER_AGENT, "hello".parse()?),
                (COOKIE, "FUZZ".parse()?),
            ]);

        assert!(builder.fuzzed_headers.get(COOKIE.as_str()).is_some());
        assert!(builder.fuzzed_headers.get(USER_AGENT.as_str()).is_none());
        assert!(builder.headers.get(COOKIE.as_str()).is_some());
        assert!(builder.headers.get(USER_AGENT.as_str()).is_some());
        Ok(())
    }
}

