use std::collections::HashMap;

use reqwest::{Client, Method, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use url::Url;

use crate::exclude_length::ExcludeContentLength;
use crate::fuzz::{FUZZ, HttpFuzzer, Result};

pub struct HttpFuzzerBuilder {
    url: Url,
    method: Method,
    headers: HeaderMap,
    status_code_blacklist: Vec<StatusCode>,
    exclude_length: ExcludeContentLength,
    fuzzed_headers: HashMap<String, String>,
}

impl HttpFuzzerBuilder {
    pub fn new() -> HttpFuzzerBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rustbuster"));

        HttpFuzzerBuilder {
            url: "http://localhost:8080".parse().unwrap(),
            headers,
            method: Method::GET,
            status_code_blacklist: Vec::new(),
            exclude_length: ExcludeContentLength::Empty,
            fuzzed_headers: HashMap::new(),
        }
    }

    pub fn build(self) -> Result<HttpFuzzer> {
        match self.validate() {
            Ok(_) => {
                let client = Client::builder()
                    .default_headers(self.headers)
                    .build()?;

                Ok(HttpFuzzer {
                    url: self.url,
                    client,
                    method: self.method,
                    status_code_blacklist: self.status_code_blacklist,
                    exclude_length: self.exclude_length,
                    fuzzed_headers: self.fuzzed_headers,
                })
            }
            Err(e) => Err(e)
        }
    }

    fn validate(&self) -> Result<()> {
        match self.url.as_str().contains(FUZZ) || self.fuzzed_headers.iter().count() > 0 {
            true => Ok(()),
            false => Err("no FUZZ keyword found".into())
        }
    }

    pub fn with_url(mut self, url: Url) -> HttpFuzzerBuilder {
        self.url = url;
        self
    }

    pub fn with_method(mut self, method: Method) -> HttpFuzzerBuilder {
        self.method = method;
        self
    }

    pub fn with_headers(mut self, headers: Vec<(HeaderName, HeaderValue)>) -> HttpFuzzerBuilder {
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

    pub fn with_status_code_blacklist(mut self, blacklist: Vec<StatusCode>) -> HttpFuzzerBuilder {
        self.status_code_blacklist = blacklist;
        self
    }

    pub fn with_exclude_length(mut self, exclude_length: ExcludeContentLength) -> HttpFuzzerBuilder {
        self.exclude_length = exclude_length;
        self
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::{COOKIE, USER_AGENT};

    use crate::fuzz::{HttpFuzzer, Result};

    #[test]
    fn error_when_no_fuzz_keyword_found() -> Result<()> {
        match HttpFuzzer::builder()
            .with_url("http://localhost:9999".parse().unwrap())
            .build() {
            Ok(_) => Err("not supposed to succeed".into()),
            Err(e) => {
                assert!(e.to_string().contains("no FUZZ keyword found"));
                Ok(())
            }
        }
    }

    #[test]
    fn headers_containing_fuzz_are_fuzzed_headers() -> Result<()> {
        let builder = HttpFuzzer::builder()
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

