use reqwest::StatusCode;

use crate::filters::body::FilterBody;
use crate::filters::content_length::FilterContentLength;
use crate::probe::ProbeResponse;

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
        let ignore_response = self.filter_status_codes.contains(&response.status_code()) ||
            self.filter_content_length.matches(response.body().len()) ||
            self.filter_body.matches(&response.body());

        return match ignore_response {
            true => None,
            false => Some(response)
        };
    }
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use crate::filters::body::FilterBody;
    use crate::filters::content_length::FilterContentLength;
    use crate::filters::response_filter::ProbeResponseFilters;
    use crate::probe::ProbeResponse;

    #[test]
    fn filter_none_matches_returns_response() -> Result<(), String> {
        let filters = ProbeResponseFilters::new(
            vec![StatusCode::NOT_FOUND],
            FilterContentLength::Empty,
            FilterBody::Empty,
        );

        let response = ProbeResponse::new(
            String::default(),
            "url".to_string(),
            StatusCode::OK,
            "hello".to_string(),
        );

        match filters.filter(response) {
            None => Err("expected response".to_string()),
            Some(r) => {
                assert_eq!(r.status_code(), StatusCode::OK);
                assert_eq!(r.body().len(), 5);
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

        let response = ProbeResponse::new(
            String::default(),
            "url".to_string(),
            StatusCode::NOT_FOUND,
            "".to_string(),
        );

        assert_eq!(filters.filter(response), None);
    }

    #[test]
    fn filter_ignores_content_length() {
        let filters = ProbeResponseFilters::new(
            Vec::new(),
            FilterContentLength::Separate(vec![35usize]),
            FilterBody::Empty,
        );

        let response = ProbeResponse::new(
            String::default(),
            "url".to_string(),
            StatusCode::NOT_FOUND,
            "qwertyuioplkjhgfdsazxcvbnmlkpoiujyh".to_string(),
        );

        assert_eq!(filters.filter(response), None);
    }

    #[test]
    fn filter_body_contains_is_ignored() {
        let filters = ProbeResponseFilters::new(
            Vec::new(),
            FilterContentLength::Empty,
            FilterBody::Text("strange word!".to_string()),
        );

        let response = ProbeResponse::new(
            String::default(),
            "url".to_string(),
            StatusCode::NOT_FOUND,
            "this contains a strange word!".to_string(),
        );

        assert_eq!(filters.filter(response), None);
    }
}
