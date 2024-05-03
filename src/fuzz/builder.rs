use std::collections::HashMap;

use reqwest::{Client, Method, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT};

use crate::exclude_length::ExcludeContentLength;
use crate::fuzz::{HttpFuzzer, Result};

pub struct HttpFuzzerBuilder {
    method: Method,
    headers: HeaderMap,
    status_code_blacklist: Vec<StatusCode>,
    exclude_length: ExcludeContentLength,
}

impl HttpFuzzerBuilder {
    pub fn new() -> HttpFuzzerBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rustbuster"));

        HttpFuzzerBuilder {
            headers,
            method: Method::GET,
            status_code_blacklist: Vec::new(),
            exclude_length: ExcludeContentLength::Empty,
        }
    }

    pub fn build(self) -> Result<HttpFuzzer> {
        let client = Client::builder()
            .default_headers(self.headers)
            .build()?;

        Ok(HttpFuzzer {
            client,
            method: self.method,
            status_code_blacklist: self.status_code_blacklist,
            exclude_length: self.exclude_length,
        })
    }

    pub fn with_method(mut self, method: Method) -> HttpFuzzerBuilder {
        self.method = method;
        self
    }

    pub fn with_headers(mut self, headers: Vec<(HeaderName, HeaderValue)>) -> Result<HttpFuzzerBuilder> {
        self.headers.extend(headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<HeaderName, HeaderValue>>());
        Ok(self)
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
