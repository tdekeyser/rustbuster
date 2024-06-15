use std::error::Error;
use std::time::Duration;

use reqwest::StatusCode;
use tokio::time;

use crate::filters::{FilterBody, FilterContentLength};
use crate::probe::{HttpProbe, ProbeResponse};
use crate::words::Wordlist;

mod progress_bar;

pub struct HttpFuzzer {
    http_probe: HttpProbe,
    filters: ProbeResponseFilters,
    delay: Option<u64>,
    verbose: bool,
}

impl HttpFuzzer {
    pub fn new(http_probe: HttpProbe,
               filters: ProbeResponseFilters,
               delay: f32,
               verbose: bool) -> Self {
        let delay = match delay {
            0.0 => None,
            _ => Some((delay * 1000.0) as u64)
        };
        Self { http_probe, filters, delay, verbose }
    }

    pub async fn brute_force(&self, wordlist: Wordlist) -> Result<(), Box<dyn Error>> {
        let pb = progress_bar::new(wordlist.len() as u64);

        for word in wordlist.iter() {
            pb.inc(1);

            let r = self.http_probe.probe(&word).await?;

            if let Some(response) = self.filters.filter(r) {
                pb.suspend(|| println!("{}", response.display(self.verbose)))
            }

            match self.delay {
                Some(delay) => time::sleep(Duration::from_millis(delay)).await,
                None => ()
            }
        }

        Ok(())
    }
}

pub struct ProbeResponseFilters {
    filter_status_codes: Vec<StatusCode>,
    filter_content_length: FilterContentLength,
    filter_body: FilterBody,
}

impl ProbeResponseFilters {
    pub fn new(filter_status_codes: Vec<StatusCode>,
               filter_content_length: FilterContentLength,
               filter_body: FilterBody) -> Self {
        Self { filter_status_codes, filter_content_length, filter_body }
    }
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


#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use crate::filters::{FilterBody, FilterContentLength};
    use crate::fuzz::ProbeResponseFilters;
    use crate::probe::ProbeResponse;

    #[test]
    fn filter_none_matches_returns_response() -> Result<(), String> {
        let filters = ProbeResponseFilters::new(
            vec![StatusCode::NOT_FOUND],
            FilterContentLength::Empty,
            FilterBody::Empty,
        );

        let response = ProbeResponse {
            request_url: "url".to_string(),
            status_code: StatusCode::OK,
            content_length: 50,
            body: "".to_string(),
        };

        match filters.filter(response) {
            None => Err("expected response".to_string()),
            Some(r) => {
                assert_eq!(r.status_code, StatusCode::OK);
                assert_eq!(r.content_length, 50);
                Ok(())
            }
        }
    }

    #[test]
    fn filter_ignores_status_codes() {
        let filters = ProbeResponseFilters::new(
            vec![StatusCode::NOT_FOUND],
            FilterContentLength::Empty,
            FilterBody::Empty,
        );

        let response = ProbeResponse {
            request_url: "url".to_string(),
            status_code: StatusCode::NOT_FOUND,
            content_length: 50,
            body: "".to_string(),
        };

        assert_eq!(filters.filter(response), None);
    }

    #[test]
    fn filter_ignores_content_length() {
        let filters = ProbeResponseFilters::new(
            Vec::new(),
            FilterContentLength::Separate(vec![35]),
            FilterBody::Empty,
        );

        let response = ProbeResponse {
            request_url: "url".to_string(),
            status_code: StatusCode::NOT_FOUND,
            content_length: 35,
            body: "".to_string(),
        };

        assert_eq!(filters.filter(response), None);
    }

    #[test]
    fn filter_body_contains_is_ignored() {
        let filters = ProbeResponseFilters::new(
            Vec::new(),
            FilterContentLength::Separate(vec![35]),
            FilterBody::Text("strange word!".to_string()),
        );

        let response = ProbeResponse {
            request_url: "url".to_string(),
            status_code: StatusCode::NOT_FOUND,
            content_length: 35,
            body: "this contains a strange word!".to_string(),
        };

        assert_eq!(filters.filter(response), None);
    }
}