use std::fmt::{Display, Formatter};

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
