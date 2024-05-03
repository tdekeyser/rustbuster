use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;

use reqwest::{Client, Method, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use url::Url;

use crate::exclude_length::ExcludeContentLength;
use crate::progress_bar;

type FuzzResult<T> = Result<T, Box<dyn Error>>;

const FUZZ: &'static str = "FUZZ";

struct HttpResponse {
    status_code: StatusCode,
    content_length: u32,
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:>10}) [Size: {:?}]", self.status_code, self.content_length)
    }
}

pub struct HttpFuzzer {
    client: Client,
    method: Method,
    status_code_blacklist: Vec<StatusCode>,
    exclude_length: ExcludeContentLength,
}

impl HttpFuzzer {
    pub fn builder() -> HttpFuzzerBuilder {
        HttpFuzzerBuilder::new()
    }

    pub async fn brute_force(&self, url: &Url, wordlist: &PathBuf) -> FuzzResult<()> {
        let wordlist = fs::read_to_string(wordlist).expect("file not found");
        let pb = progress_bar::new(wordlist.lines().count() as u64);

        for word in wordlist.lines() {
            match self.probe(url, word).await? {
                Some(response) => pb.println(format!("/{:<30} {}", word, response)),
                None => ()
            }

            pb.inc(1);
        }
        Ok(())
    }

    async fn probe(&self, url: &Url, word: &str) -> FuzzResult<Option<HttpResponse>> {
        let request_url = url.as_str().replace(FUZZ, word);

        let response = self.client.request(self.method.clone(), request_url)
            .send()
            .await?;

        let status_code = response.status();
        let content_length = response.text().await?.len() as u32;

        let ignore_result = self.status_code_blacklist.contains(&status_code) ||
            self.exclude_length.matches(content_length);

        match ignore_result {
            true => Ok(None),
            false => Ok(Some(HttpResponse { status_code, content_length }))
        }
    }
}

pub struct HttpFuzzerBuilder {
    method: Method,
    headers: HeaderMap,
    status_code_blacklist: Vec<StatusCode>,
    exclude_length: ExcludeContentLength,
}

impl HttpFuzzerBuilder {
    fn new() -> HttpFuzzerBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rustbuster"));

        HttpFuzzerBuilder {
            headers,
            method: Method::GET,
            status_code_blacklist: Vec::new(),
            exclude_length: ExcludeContentLength::Empty,
        }
    }

    pub fn build(self) -> FuzzResult<HttpFuzzer> {
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

    pub fn with_headers(mut self, headers: Vec<(HeaderName, HeaderValue)>) -> FuzzResult<HttpFuzzerBuilder> {
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


#[cfg(test)]
mod tests {
    use reqwest::StatusCode;
    use url::Url;

    use crate::exclude_length::ExcludeContentLength;
    use crate::fuzz::{FuzzResult, HttpFuzzer};

    #[tokio::test]
    async fn fuzzer_gets_response() -> FuzzResult<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/hello")
            .with_status(200)
            .with_body("hello")
            .create_async().await;

        let fuzzer = HttpFuzzer::builder().build()?;

        let url = Url::parse(&format!("{}/FUZZ", server.url()).as_str())?;

        match fuzzer.probe(&url, "hello").await? {
            Some(r) => {
                assert_eq!(r.status_code, StatusCode::OK);
                assert_eq!(r.content_length, 5);
                Ok(())
            }
            None => Err("expected a response".into())
        }
    }

    #[tokio::test]
    async fn fuzzer_ignores_status_codes() -> FuzzResult<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/non-existing").with_status(404).create_async().await;

        let fuzzer = HttpFuzzer::builder()
            .with_status_code_blacklist(vec![StatusCode::NOT_FOUND])
            .build()?;

        let url = Url::parse(&format!("{}/FUZZ", server.url()).as_str())?;

        match fuzzer.probe(&url, "non-existing").await? {
            Some(r) => Err(format!("{:}", r).into()),
            None => Ok(())
        }
    }

    #[tokio::test]
    async fn fuzzer_ignores_content_length() -> FuzzResult<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/len")
            .with_body("This body is exactly 35 chars long.")
            .create_async()
            .await;

        let fuzzer = HttpFuzzer::builder()
            .with_exclude_length(ExcludeContentLength::Separate(vec!(35)))
            .build()?;

        let url = Url::parse(&format!("{}/FUZZ", server.url()).as_str())?;

        match fuzzer.probe(&url, "len").await? {
            Some(r) => Err(format!("{:}", r).into()),
            None => Ok(())
        }
    }
}
