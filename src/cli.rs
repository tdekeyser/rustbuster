use clap::Parser;
use http::{Method, StatusCode};

use crate::filters::{FilterBody, FilterContentLength};
use crate::Result;

/// Imitation of Gobuster/ffuf in Rust.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The target URL
    #[arg(short, long)]
    pub url: String,

    /// Path to the wordlist.
    #[arg(short, long)]
    pub wordlist: std::path::PathBuf,

    /// File extensions to search for, e.g. json,xml
    #[arg(short = 'x', long, value_delimiter = ',', default_value = "")]
    pub extensions: Vec<String>,

    /// Use the following HTTP method
    #[arg(short, long, default_value = "GET")]
    pub method: Method,

    /// Custom headers; use the format "Header1: Content1, Header2: Content2"
    #[arg(short = 'H', long, value_delimiter = ',', value_parser = parse_headers, required = false)]
    pub headers: Vec<(String, String)>,

    /// Request body
    #[arg(short, long, default_value = "")]
    pub body: String,

    /// Delay between requests, in seconds
    #[arg(short, long, default_value_t = 0.0)]
    pub delay: f32,

    /// Number of threads, default 10.
    #[arg(short, long, default_value_t = 10)]
    pub threads: usize,

    /// Status code that will be ignored, e.g. 404,500
    #[arg(long, value_delimiter = ',', default_value = "404")]
    pub filter_status_codes: Vec<StatusCode>,

    /// Content lengths that will be ignored, e.g. 20,300, or a range, e.g. 20-300
    #[arg(long, default_value_t = FilterContentLength::Empty)]
    pub filter_content_length: FilterContentLength,

    /// Ignore if text appears in the response body
    #[arg(long, default_value_t = FilterBody::Empty)]
    pub filter_body: FilterBody,

    /// Verbose output including response status code, content length, etc.
    #[arg(short, long)]
    pub verbose: bool,
}

fn parse_headers(s: &str) -> Result<(String, String)> {
    let pos = s
        .find(':')
        .ok_or_else(|| format!("invalid content for `{s}`: format 'Header1: Content1, Header2: Content2'"))?;
    Ok((s[..pos].trim().to_string(), s[pos + 1..].trim().to_string()))
}


#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::cli::parse_headers;

    #[test]
    fn parse_key_val_parses_colon() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let (key, val) = parse_headers("user-agent: rustbuster")?;

        assert_eq!(key, "user-agent");
        assert_eq!(val, "rustbuster");
        Ok(())
    }

    #[test]
    #[should_panic]
    fn parse_headers_invalid_header_name() {
        parse_headers("User Agent; hello!,").unwrap();
    }
}
