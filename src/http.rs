use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::PathBuf;

use reqwest::{Client, Error, Method, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use url::Url;

use crate::exclude_length::ExcludeContentLength;
use crate::progress_bar;

pub struct HttpBrute {
    client: Client,
    method: Method,
    status_code_blacklist: Vec<StatusCode>,
    exclude_length: ExcludeContentLength,
}

#[derive(Debug)]
pub struct HttpResponse {
    status_code: StatusCode,
    content_length: u32,
}

pub struct HttpError(pub String);

impl HttpBrute {
    pub fn builder() -> HttpBruteBuilder {
        HttpBruteBuilder::new()
    }

    pub async fn brute_force(&self, url: &Url, wordlist: &PathBuf) -> Result<(), HttpError> {
        let wordlist = fs::read_to_string(wordlist).expect("file not found");
        let pb = progress_bar::new(wordlist.lines().count() as u64);

        for word in wordlist.lines() {
            let request_url = format!("{}{}", url, word);

            match self.probe(&request_url).await? {
                Some(response) => pb.println(format!("/{:<30} {}", word, response)),
                None => ()
            }

            pb.inc(1);
        }
        Ok(())
    }

    async fn probe(&self, url: &str) -> Result<Option<HttpResponse>, HttpError> {
        let response = self.client.request(self.method.clone(), url).send().await?;
        let status_code = response.status();
        let content_length = response.text().await.unwrap().len() as u32;

        let ignore_result = self.status_code_blacklist.contains(&status_code) ||
            self.exclude_length.matches(content_length);

        match ignore_result {
            true => Ok(None),
            false => Ok(Some(HttpResponse { status_code, content_length }))
        }
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:>10}) [Size: {:?}]", self.status_code, self.content_length)
    }
}


impl From<Error> for HttpError {
    fn from(e: Error) -> Self {
        HttpError(e.to_string())
    }
}

impl Debug for HttpError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct HttpBruteBuilder {
    method: Method,
    headers: HeaderMap,
    status_code_blacklist: Vec<StatusCode>,
    exclude_length: ExcludeContentLength,
}

impl HttpBruteBuilder {
    fn new() -> HttpBruteBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rustbuster"));

        HttpBruteBuilder {
            headers,
            method: Method::GET,
            status_code_blacklist: Vec::new(),
            exclude_length: ExcludeContentLength::Empty,
        }
    }

    pub fn build(self) -> Result<HttpBrute, HttpError> {
        let client = Client::builder()
            .default_headers(self.headers)
            .build()?;

        Ok(HttpBrute {
            client,
            method: self.method,
            status_code_blacklist: self.status_code_blacklist,
            exclude_length: self.exclude_length,
        })
    }

    pub fn with_method(mut self, method: Method) -> HttpBruteBuilder {
        self.method = method;
        self
    }

    pub fn with_headers(mut self, headers: Vec<(HeaderName, HeaderValue)>) -> Result<HttpBruteBuilder, HttpError> {
        self.headers.extend(headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<HeaderName, HeaderValue>>());
        Ok(self)
    }

    pub fn with_status_code_blacklist(mut self, blacklist: Vec<StatusCode>) -> HttpBruteBuilder {
        self.status_code_blacklist = blacklist;
        self
    }

    pub fn with_exclude_length(mut self, exclude_length: ExcludeContentLength) -> HttpBruteBuilder {
        self.exclude_length = exclude_length;
        self
    }
}


#[cfg(test)]
mod tests {
    use reqwest::header::{AUTHORIZATION, HeaderValue, USER_AGENT};
    use reqwest::StatusCode;

    use crate::http::{HttpBrute, HttpError};

    #[tokio::test]
    async fn http_client_ignores_status_codes() -> Result<(), HttpError> {
        let http_client = HttpBrute::builder()
            .with_status_code_blacklist(vec![StatusCode::NOT_FOUND])
            .build()?;

        match http_client.probe("http://localhost:8080/helo").await? {
            Some(r) => Err(HttpError(format!("{:?}", r))),
            None => Ok(())
        }
    }

    #[test]
    fn http_client_builder_maps_headers() -> Result<(), HttpError> {
        let headers = vec![
            (USER_AGENT, HeaderValue::from_static("rustbuster")),
            (AUTHORIZATION, HeaderValue::from_static("Bearer 1234")),
        ];

        let _client = HttpBrute::builder()
            .with_headers(headers)?
            .build()?;

        Ok(())
    }
}
