use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum ExcludeContentLength {
    Separate(Vec<u32>),
    Range(u32, u32),
    Empty,
}

impl Display for ExcludeContentLength {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


impl From<&str> for ExcludeContentLength {
    fn from(value: &str) -> Self {
        if value.contains("-") {
            return Self::from_nums(value.split("-")
                .map(|v| v.parse::<u32>())
                .flatten()
                .collect());
        }

        ExcludeContentLength::Separate(value.split(",")
            .map(|v| v.parse::<u32>())
            .flatten()
            .collect())
    }
}


impl ExcludeContentLength {
    pub fn matches(&self, length: u32) -> bool {
        match self {
            ExcludeContentLength::Empty => false,
            ExcludeContentLength::Separate(v) => v.contains(&length),
            ExcludeContentLength::Range(a, b) => a <= &length && &length <= b
        }
    }

    fn from_nums(nums: Vec<u32>) -> ExcludeContentLength {
        let nums: [u32; 2] = nums.try_into()
            .unwrap_or_else(|_| panic!("expected 2 values in content length range"));

        if nums[0] < nums[1] {
            return ExcludeContentLength::Range(nums[0], nums[1]);
        }

        panic!("invalid range")
    }
}

#[cfg(test)]
mod tests {
    use crate::exclude_length::ExcludeContentLength;

    #[test]
    fn exclude_lengths_from_str_separate() {
        let exclude_lengths = ExcludeContentLength::from("30,12");
        assert_eq!(exclude_lengths, ExcludeContentLength::Separate(vec! {30, 12}));
    }

    #[test]
    fn exclude_lengths_from_str_separate_ignores_other() {
        let exclude_lengths = ExcludeContentLength::from("20,2,ab");
        assert_eq!(exclude_lengths, ExcludeContentLength::Separate(vec! {20, 2}));
    }

    #[test]
    fn exclude_lengths_from_str_range() {
        let exclude_lengths = ExcludeContentLength::from("20-300");
        assert_eq!(exclude_lengths, ExcludeContentLength::Range(20, 300));
    }

    #[test]
    #[should_panic]
    fn exclude_lengths_from_str_err_range() {
        let _ = ExcludeContentLength::from("300-20");
    }

    #[test]
    fn matches_empty() {
        assert!(!ExcludeContentLength::Empty.matches(4))
    }

    #[test]
    fn matches_separate() {
        assert!(ExcludeContentLength::Separate(vec![200, 40, 404]).matches(404));
        assert!(!ExcludeContentLength::Separate(vec![200, 40, 404]).matches(500));
    }

    #[test]
    fn matches_range_inclusive() {
        assert!(!ExcludeContentLength::Range(200, 404).matches(500));
        assert!(ExcludeContentLength::Range(200, 500).matches(500));
    }
}