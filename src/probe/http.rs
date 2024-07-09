use std::collections::HashMap;

use reqwest::{Client, Method, redirect, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use reqwest::Url;

use crate::{Error, Result};
use crate::probe::FUZZ;

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
        let request_url = self.url.replace(FUZZ, word);
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

pub struct HttpProbeBuilder {
    url: String,
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
        match self.url.contains(FUZZ) || self.fuzzed_headers.iter().count() > 0 {
            true => Ok(()),
            false => Err(Error::FuzzKeywordNotFound)
        }
    }

    pub fn with_url(mut self, url: String) -> HttpProbeBuilder {
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
    use reqwest::header::{COOKIE, USER_AGENT};
    use reqwest::StatusCode;

    use crate::probe::HttpProbe;
    use crate::Result;

    #[test]
    fn error_when_no_fuzz_keyword_found() -> Result<()> {
        match HttpProbe::builder()
            .with_url("http://localhost:9999".parse().unwrap())
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
    fn no_error_when_fuzz_keyword_found() -> Result<()> {
        match HttpProbe::builder()
            .with_url("http://FUZZ.localhost:9999".parse().unwrap())
            .build() {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }

    #[test]
    fn no_error_when_fuzz_keyword_found_in_dir() -> Result<()> {
        match HttpProbe::builder()
            .with_url("http://localhost:9999/FUZZ".parse().unwrap())
            .build() {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }

    #[test]
    fn headers_containing_fuzz_are_fuzzed_headers() -> Result<()> {
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
