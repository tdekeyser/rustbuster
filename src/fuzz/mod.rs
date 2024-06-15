use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::Duration;

use reqwest::{Client, Method, StatusCode};
use reqwest::header::{HeaderMap, HeaderName};
use tokio::time;
use url::Url;

use crate::fuzz::filters::{FilterBody, FilterContentLength};
use crate::words::Wordlist;

pub mod filters;
mod builder;
mod progress_bar;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const FUZZ: &'static str = "FUZZ";

struct ProbeResponse {
    request_url: String,
    status_code: StatusCode,
    content_length: u32,
    body: String,
}

impl Display for ProbeResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:>10}) [Size: {:?}]", self.status_code, self.content_length)
    }
}

pub struct ProbeResponseFilters {
    filter_status_codes: Vec<StatusCode>,
    filter_content_length: FilterContentLength,
    filter_body: FilterBody,
}

impl ProbeResponseFilters {
    fn filter(&self, response: ProbeResponse) -> Option<ProbeResponse> {
        let ProbeResponse {
            request_url: ref _request_url,
            ref status_code,
            content_length,
            ref body
        } = response;

        let check = self.filter_status_codes.contains(status_code) ||
            self.filter_content_length.matches(content_length) ||
            self.filter_body.matches(body);

        return match check {
            true => None,
            false => Some(response)
        };
    }
}

pub struct HttpFuzzer {
    url: Url,
    client: Client,
    method: Method,
    delay: Option<u64>,
    response_filters: ProbeResponseFilters,
    fuzzed_headers: HashMap<String, String>,
}

impl HttpFuzzer {
    pub fn builder() -> builder::HttpFuzzerBuilder {
        builder::HttpFuzzerBuilder::new()
    }

    pub async fn brute_force(&self, wordlist: Wordlist) -> Result<()> {
        let pb = progress_bar::new(wordlist.len() as u64);

        for word in wordlist.iter() {
            pb.inc(1);
            match self.probe(&word).await? {
                Some(response) => pb.println(format!("{}", response.request_url)),
                None => ()
            }
            match self.delay {
                Some(delay) => time::sleep(Duration::from_millis(delay)).await,
                None => ()
            }
        }

        Ok(())
    }

    async fn probe(&self, word: &str) -> Result<Option<ProbeResponse>> {
        let request_url = self.url.as_str().replace(FUZZ, word);
        let extra_headers = self.replace_keyword_in_headers(word)?;

        let response = self.client
            .request(self.method.clone(), &request_url)
            .headers(extra_headers)
            .send()
            .await?;

        let status_code = response.status();
        let body = response.text().await.or::<reqwest::Error>(Ok("".to_string()))?;
        let content_length = body.len() as u32;

        Ok(self.response_filters.filter(ProbeResponse {
            request_url,
            status_code,
            content_length,
            body,
        }))
    }

    fn replace_keyword_in_headers(&self, word: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        for (k, v) in self.fuzzed_headers.iter() {
            let key = k.replace(FUZZ, word);
            let value = v.replace(FUZZ, word);
            headers.insert(HeaderName::from_bytes(key.as_bytes())?, value.parse()?);
        }
        Ok(headers)
    }
}


#[cfg(test)]
mod tests {
    use reqwest::header::USER_AGENT;
    use reqwest::StatusCode;
    use url::Url;

    use crate::fuzz::{HttpFuzzer, Result};
    use crate::fuzz::filters::FilterContentLength;

    #[tokio::test]
    async fn fuzzer_gets_response() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/hello")
            .with_status(200)
            .with_body("hello")
            .create_async().await;

        let url = Url::parse(&format!("{}/FUZZ", server.url()).as_str())?;

        let fuzzer = HttpFuzzer::builder()
            .with_url(url)
            .build()?;

        match fuzzer.probe("hello").await? {
            Some(r) => {
                assert_eq!(r.status_code, StatusCode::OK);
                assert_eq!(r.content_length, 5);
                Ok(())
            }
            None => Err("expected a response".into())
        }
    }

    #[tokio::test]
    async fn fuzzer_ignores_status_codes() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/non-existing").with_status(404).create_async().await;

        let url = Url::parse(&format!("{}/FUZZ", server.url()).as_str())?;

        let fuzzer = HttpFuzzer::builder()
            .with_url(url)
            .with_status_codes_filter(vec![StatusCode::NOT_FOUND])
            .build()?;

        match fuzzer.probe("non-existing").await? {
            Some(r) => Err(format!("{:}", r).into()),
            None => Ok(())
        }
    }

    #[tokio::test]
    async fn fuzzer_ignores_content_length() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/len")
            .with_body("This body is exactly 35 chars long.")
            .create_async()
            .await;

        let url = Url::parse(&format!("{}/FUZZ", server.url()).as_str())?;

        let fuzzer = HttpFuzzer::builder()
            .with_url(url)
            .with_content_length_filter(FilterContentLength::Separate(vec!(35)))
            .build()?;

        match fuzzer.probe("len").await? {
            Some(r) => Err(format!("{:}", r).into()),
            None => Ok(())
        }
    }

    #[tokio::test]
    async fn fuzzer_keyword_in_headers() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/do-fuzz")
            .match_header(USER_AGENT.as_str(), "fill-to-header")
            .create_async()
            .await;

        let url = Url::parse(&format!("{}/do-fuzz", server.url()).as_str())?;

        let fuzzer = HttpFuzzer::builder()
            .with_url(url)
            .with_headers(vec![(USER_AGENT, "FUZZ".parse()?)])
            .build()?;

        match fuzzer.probe("fill-to-header").await? {
            Some(r) => {
                Ok(assert_eq!(r.status_code, StatusCode::OK))
            }
            None => Err("expected response".into())
        }
    }
}
