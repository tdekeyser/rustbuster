use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use reqwest::{Client, Error, Method, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT};

use crate::exclude_length::ExcludeContentLength;

pub struct HttpClient {
    client: Client,
    status_code_blacklist: Vec<StatusCode>,
    exclude_length: ExcludeContentLength,
}

pub struct HttpClientBuilder {
    headers: HeaderMap,
    status_code_blacklist: Vec<StatusCode>,
    exclude_length: ExcludeContentLength,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub content_length: u32,
}

pub struct HttpError(pub String);

impl Display for HttpResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:>10}) [Size: {:?}]", self.status_code, self.content_length)
    }
}

impl HttpClient {
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }

    pub async fn probe(&self, url: &str) -> Result<Option<HttpResponse>, HttpError> {
        let response = self.client.request(Method::GET, url).send().await?;
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

impl HttpClientBuilder {
    fn new() -> HttpClientBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rustbuster"));

        HttpClientBuilder {
            headers,
            status_code_blacklist: Vec::new(),
            exclude_length: ExcludeContentLength::Empty,
        }
    }

    pub fn build(self) -> Result<HttpClient, HttpError> {
        let client = Client::builder()
            .default_headers(self.headers)
            .build()?;

        Ok(HttpClient {
            client,
            status_code_blacklist: self.status_code_blacklist,
            exclude_length: self.exclude_length,
        })
    }

    pub fn with_headers(mut self, headers: Vec<(HeaderName, HeaderValue)>) -> Result<HttpClientBuilder, HttpError> {
        self.headers.extend(headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<HeaderName, HeaderValue>>());
        Ok(self)
    }

    pub fn with_status_code_blacklist(mut self, blacklist: Vec<StatusCode>) -> HttpClientBuilder {
        self.status_code_blacklist = blacklist;
        self
    }

    pub fn with_exclude_length(mut self, exclude_length: ExcludeContentLength) -> HttpClientBuilder {
        self.exclude_length = exclude_length;
        self
    }
}


#[cfg(test)]
mod tests {
    use reqwest::header::{AUTHORIZATION, HeaderValue, USER_AGENT};
    use reqwest::StatusCode;

    use crate::http::{HttpClient, HttpError};

    #[tokio::test]
    async fn http_client_ignores_status_codes() -> Result<(), HttpError> {
        let http_client = HttpClient::builder()
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

        let _client = HttpClient::builder()
            .with_headers(headers)?
            .build()?;

        Ok(())
    }
}
