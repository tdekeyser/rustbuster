use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use reqwest::{Client, Method, StatusCode};
use reqwest::header::{HeaderMap, HeaderName};
use tokio::time;
use url::Url;

use crate::filters::{FilterBody, FilterContentLength};
use crate::words::Wordlist;

mod builder;
mod progress_bar;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const FUZZ: &'static str = "FUZZ";

#[derive(Debug)]
struct ProbeResponse {
    request_url: String,
    status_code: StatusCode,
    content_length: u32,
    body: String,
}

pub struct ProbeResponseFilters {
    filter_status_codes: Vec<StatusCode>,
    filter_content_length: FilterContentLength,
    filter_body: FilterBody,
}

impl ProbeResponseFilters {
    fn filter(&self, response: ProbeResponse) -> Option<ProbeResponse> {
        let ignore_response = self.filter_status_codes.contains(&response.status_code) ||
            self.filter_content_length.matches(response.content_length) ||
            self.filter_body.matches(&response.body);

        return match ignore_response {
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
    verbose: bool,
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
                Some(response) => pb.suspend(|| self.print(response)),
                None => ()
            }
            match self.delay {
                Some(delay) => time::sleep(Duration::from_millis(delay)).await,
                None => ()
            }
        }

        Ok(())
    }

    fn print(&self, response: ProbeResponse) {
        match self.verbose {
            true => println!("{:<30} ({:>10}) [Size: {:?}]",
                             Url::parse(response.request_url.as_str())
                                 .map(|u| u.path().to_owned())
                                 .unwrap_or_default(),
                             response.status_code,
                             response.content_length),
            false => println!("{}", response.request_url),
        }
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

    use crate::filters::FilterContentLength;
    use crate::fuzz::{HttpFuzzer, Result};

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
            Some(r) => Err(format!("{:?}", r).into()),
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
            Some(r) => Err(format!("{:?}", r).into()),
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
