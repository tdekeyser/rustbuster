use std::collections::HashMap;

use http::{Method, StatusCode};
use http::header::{HeaderMap, HeaderName};
use reqwest::{Client, redirect};

use crate::{Error, Result};
use crate::probe::FUZZ;

#[derive(Debug, PartialEq)]
pub struct ProbeResponse {
    word: String,
    request_url: String,
    status_code: StatusCode,
    body: String,
}

impl ProbeResponse {
    pub fn display(&self, verbose: bool) -> String {
        if verbose {
            return format!("{:<30} ({:>10}) [Size: {:?}]",
                           self.word,
                           self.status_code,
                           self.body.len());
        }
        self.request_url.clone()
    }

    pub fn new(word: String,
               request_url: String,
               status_code: StatusCode,
               body: String) -> Self {
        Self { word, request_url, status_code, body }
    }

    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

pub struct HttpProbe {
    url: String,
    client: Client,
    method: Method,
    fuzzed_headers: HashMap<String, String>,
    body: String,
}

impl HttpProbe {
    pub fn builder() -> HttpProbeBuilder {
        HttpProbeBuilder::new()
    }

    pub async fn probe(&self, word: &str) -> Result<ProbeResponse> {
        let request_url = self.url.replace(FUZZ, word);
        let extra_headers = self.replace_keyword_in_headers(word)?;
        let body = self.body.replace(FUZZ, word);

        let response = self.client
            .request(self.method.clone(), &request_url)
            .headers(extra_headers)
            .body(body)
            .send()
            .await?;

        let status_code = response.status();
        let body = response.text().await.ok().unwrap_or_default();

        Ok(ProbeResponse::new(
            word.to_string(),
            request_url,
            status_code,
            body,
        ))
    }

    fn replace_keyword_in_headers(&self, word: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        for (k, v) in self.fuzzed_headers.iter() {
            headers.insert(
                HeaderName::from_bytes(k.replace(FUZZ, word).as_bytes())?,
                v.replace(FUZZ, word).parse()?,
            );
        }
        Ok(headers)
    }
}

pub struct HttpProbeBuilder {
    url: String,
    method: Method,
    headers: HeaderMap,
    fuzzed_headers: HashMap<String, String>,
    body: String,
}

impl HttpProbeBuilder {
    pub fn new() -> HttpProbeBuilder {
        HttpProbeBuilder {
            url: "http://localhost:8080/FUZZ".parse().unwrap(),
            method: Method::GET,
            headers: HeaderMap::new(),
            fuzzed_headers: HashMap::new(),
            body: String::default(),
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
            body: self.body,
        })
    }

    fn validate(&self) -> Result<()> {
        let conditions = [
            self.url.contains(FUZZ),
            self.body.contains(FUZZ),
            !self.fuzzed_headers.is_empty()
        ];

        if conditions.iter().any(|&c| c) {
            Ok(())
        } else {
            Err(Error::FuzzKeywordNotFound)
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

    pub fn with_body(mut self, body: String) -> HttpProbeBuilder {
        self.body = body;
        self
    }

    pub fn with_headers(mut self, headers: Vec<(String, String)>) -> Result<HttpProbeBuilder> {
        for (k, v) in headers {
            if format!("{:?}{:?}", k, v).contains(FUZZ) {
                self.fuzzed_headers.insert(k, v);
            } else {
                self.headers.insert(HeaderName::from_bytes(k.as_bytes())?, v.parse()?);
            }
        }
        Ok(self)
    }
}


#[cfg(test)]
mod tests {
    use http::Method;
    use reqwest::header::{COOKIE, USER_AGENT};
    use reqwest::StatusCode;

    use crate::probe::{FUZZ, HttpProbe};
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
                (USER_AGENT.to_string(), "hello".to_string()),
                (COOKIE.to_string(), "FUZZ".to_string()),
            ])?;

        assert!(builder.fuzzed_headers.get(COOKIE.as_str()).is_some());
        assert!(builder.fuzzed_headers.get(USER_AGENT.as_str()).is_none());
        assert!(builder.headers.get(COOKIE.as_str()).is_none());
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

        let fuzzer = HttpProbe::builder()
            .with_url(format!("{}/FUZZ", server.url()))
            .build()?;

        let r = fuzzer.probe("hello").await?;

        assert_eq!(r.status_code(), StatusCode::OK);
        assert_eq!(r.body().len(), 5);
        Ok(())
    }

    #[tokio::test]
    async fn fuzzer_keyword_in_header_value() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/do-fuzz")
            .match_header(USER_AGENT.as_str(), "fill-to-header")
            .create_async()
            .await;

        let http_probe = HttpProbe::builder()
            .with_url(format!("{}/do-fuzz", server.url()))
            .with_headers(vec![(USER_AGENT.to_string(), "FUZZ".to_string())])?
            .build()?;

        let r = http_probe.probe("fill-to-header").await?;

        assert_eq!(r.status_code(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn fuzzer_keyword_in_header_name() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/do-fuzz")
            .match_header("X-replaced-word", "100")
            .create_async()
            .await;

        let http_probe = HttpProbe::builder()
            .with_url(format!("{}/do-fuzz", server.url()))
            .with_headers(vec![(FUZZ.to_string(), "100".to_string())])?
            .build()?;

        let r = http_probe.probe("X-replaced-word").await?;

        assert_eq!(r.status_code(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn fuzzer_keyword_in_body() -> Result<()> {
        let mut server = mockito::Server::new_async().await;
        server.mock("POST", "/do-fuzz")
            .match_body("Hello X-replaced-word")
            .create_async()
            .await;

        let http_probe = HttpProbe::builder()
            .with_method(Method::POST)
            .with_url(format!("{}/do-fuzz", server.url()))
            .with_body("Hello FUZZ".to_string())
            .build()?;

        let r = http_probe.probe("X-replaced-word").await?;

        assert_eq!(r.status_code(), StatusCode::OK);
        Ok(())
    }
}
