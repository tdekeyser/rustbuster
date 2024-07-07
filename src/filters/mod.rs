use std::fmt::{Debug, Display, Formatter};

use reqwest::StatusCode;

use crate::probe::ProbeResponse;

#[derive(Clone)]
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

    pub fn filter(&self, response: ProbeResponse) -> Option<ProbeResponse> {
        let ignore_response = self.filter_status_codes.contains(&response.status_code) ||
            self.filter_content_length.matches(response.content_length) ||
            self.filter_body.matches(&response.body);

        return match ignore_response {
            true => None,
            false => Some(response)
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FilterContentLength {
    Separate(Vec<u32>),
    Range(u32, u32),
    Empty,
}

impl Display for FilterContentLength {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&str> for FilterContentLength {
    fn from(value: &str) -> Self {
        if value.contains("-") {
            return Self::from_nums(value.split("-")
                .map(|v| v.parse::<u32>())
                .flatten()
                .collect());
        }

        FilterContentLength::Separate(value.split(",")
            .map(|v| v.parse::<u32>())
            .flatten()
            .collect())
    }
}

impl FilterContentLength {
    pub fn matches(&self, length: u32) -> bool {
        match self {
            FilterContentLength::Empty => false,
            FilterContentLength::Separate(v) => v.contains(&length),
            FilterContentLength::Range(a, b) => a <= &length && &length <= b
        }
    }

    fn from_nums(nums: Vec<u32>) -> FilterContentLength {
        let nums: [u32; 2] = nums.try_into()
            .unwrap_or_else(|_| panic!("expected 2 values in content length range"));

        if nums[0] < nums[1] {
            return FilterContentLength::Range(nums[0], nums[1]);
        }

        panic!("invalid range")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FilterBody {
    Text(String),
    Empty,
}

impl From<&str> for FilterBody {
    fn from(value: &str) -> Self {
        match value {
            "" => FilterBody::Empty,
            v => FilterBody::Text(String::from(v))
        }
    }
}

impl FilterBody {
    pub fn matches(&self, content: &str) -> bool {
        match self {
            FilterBody::Empty => false,
            FilterBody::Text(c) => content.contains(c),
        }
    }
}

impl Display for FilterBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use crate::filters::{FilterBody, FilterContentLength, ProbeResponseFilters};
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

    #[test]
    fn exclude_lengths_from_str_separate() {
        let exclude_lengths = FilterContentLength::from("30,12");
        assert_eq!(exclude_lengths, FilterContentLength::Separate(vec! {30, 12}));
    }

    #[test]
    fn exclude_lengths_from_str_separate_ignores_other() {
        let exclude_lengths = FilterContentLength::from("20,2,ab");
        assert_eq!(exclude_lengths, FilterContentLength::Separate(vec! {20, 2}));
    }

    #[test]
    fn exclude_lengths_from_str_range() {
        let exclude_lengths = FilterContentLength::from("20-300");
        assert_eq!(exclude_lengths, FilterContentLength::Range(20, 300));
    }

    #[test]
    #[should_panic]
    fn exclude_lengths_from_str_err_range() {
        let _ = FilterContentLength::from("300-20");
    }

    #[test]
    fn matches_empty() {
        assert!(!FilterContentLength::Empty.matches(4))
    }

    #[test]
    fn matches_separate() {
        assert!(FilterContentLength::Separate(vec![200, 40, 404]).matches(404));
        assert!(!FilterContentLength::Separate(vec![200, 40, 404]).matches(500));
    }

    #[test]
    fn matches_range_inclusive() {
        assert!(!FilterContentLength::Range(200, 404).matches(500));
        assert!(FilterContentLength::Range(200, 500).matches(500));
    }
}
