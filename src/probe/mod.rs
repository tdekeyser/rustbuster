use std::collections::HashMap;

use reqwest::{Client, Method, StatusCode};
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Url;

use crate::probe::builder::HttpProbeBuilder;
use crate::Result;

pub mod builder;

const FUZZ: &'static str = "FUZZ";

#[derive(Clone)]
pub struct HttpProbe {
    url: String,
    client: Client,
    method: Method,
    fuzzed_headers: HashMap<String, String>,
}

impl HttpProbe {
    pub fn builder() -> HttpProbeBuilder {
        HttpProbeBuilder::new()
    }

    pub async fn probe(&self, word: &str) -> Result<ProbeResponse> {
        let request_url = self.url.as_str().replace(FUZZ, word);
        let extra_headers = self.replace_keyword_in_headers(word)?;

        let response = self.client
            .request(self.method.clone(), &request_url)
            .headers(extra_headers)
            .send()
            .await?;

        let status_code = response.status();
        let body = response.text().await.ok().unwrap_or_default();
        let content_length = body.len() as u32;

        Ok(ProbeResponse {
            request_url,
            status_code,
            content_length,
            body,
        })
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

#[derive(Debug, PartialEq)]
pub struct ProbeResponse {
    pub request_url: String,
    pub status_code: StatusCode,
    pub content_length: u32,
    pub body: String,
}

impl ProbeResponse {
    pub fn display(&self, verbose: bool) -> String {
        if verbose {
            let url_path = Url::parse(self.request_url.as_str())
                .map(|u| u.path().to_owned())
                .unwrap_or_default();

            return format!("{:<30} ({:>10}) [Size: {:?}]",
                           url_path,
                           self.status_code,
                           self.content_length);
        }
        self.request_url.clone()
    }
}


#[cfg(test)]
mod tests {
    use reqwest::header::USER_AGENT;
    use reqwest::StatusCode;

    use crate::probe::HttpProbe;
    use crate::Result;

    #[tokio::test]
    async fn fuzzer_gets_response() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/hello")
            .with_status(200)
            .with_body("hello")
            .create_async().await;

        let url = format!("{}/FUZZ", server.url());

        let fuzzer = HttpProbe::builder()
            .with_url(url)
            .build()?;

        let r = fuzzer.probe("hello").await?;

        assert_eq!(r.status_code, StatusCode::OK);
        assert_eq!(r.content_length, 5);
        Ok(())
    }

    #[tokio::test]
    async fn fuzzer_keyword_in_headers() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/do-fuzz")
            .match_header(USER_AGENT.as_str(), "fill-to-header")
            .create_async()
            .await;

        let url = format!("{}/do-fuzz", server.url());

        let fuzzer = HttpProbe::builder()
            .with_url(url)
            .with_headers(vec![(USER_AGENT, "FUZZ".parse()?)])
            .build()?;

        let r = fuzzer.probe("fill-to-header").await?;

        assert_eq!(r.status_code, StatusCode::OK);
        Ok(())
    }
}