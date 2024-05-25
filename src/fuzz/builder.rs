use std::collections::HashMap;

use reqwest::{Client, Method, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use url::Url;

use crate::fuzz::{FUZZ, HttpFuzzer, HttpResponseFilters, Result};
use crate::fuzz::filters::{FilterBody, FilterContentLength};

pub struct HttpFuzzerBuilder {
    url: Url,
    method: Method,
    headers: HeaderMap,
    filter_status_codes: Vec<StatusCode>,
    filter_content_length: FilterContentLength,
    filter_body: FilterBody,
    fuzzed_headers: HashMap<String, String>,
}

impl HttpFuzzerBuilder {
    pub fn new() -> HttpFuzzerBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rustbuster"));

        HttpFuzzerBuilder {
            url: "http://localhost:8080/FUZZ".parse().unwrap(),
            headers,
            method: Method::GET,
            filter_status_codes: Vec::new(),
            filter_content_length: FilterContentLength::Empty,
            filter_body: FilterBody::Empty,
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
                    response_filters: HttpResponseFilters {
                        filter_status_codes: self.filter_status_codes,
                        filter_content_length: self.filter_content_length,
                        filter_body: self.filter_body,
                    },
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

    pub fn with_status_codes_filter(mut self, blacklist: Vec<StatusCode>) -> HttpFuzzerBuilder {
        self.filter_status_codes = blacklist;
        self
    }

    pub fn with_content_length_filter(mut self, exclude_length: FilterContentLength) -> HttpFuzzerBuilder {
        self.filter_content_length = exclude_length;
        self
    }

    pub fn with_body_filter(mut self, filter_body: FilterBody) -> HttpFuzzerBuilder {
        self.filter_body = filter_body;
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

