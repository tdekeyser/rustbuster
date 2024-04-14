use std::error::Error;

use clap::{Parser, Subcommand};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::StatusCode;
use url::Url;

use crate::exclude_length::ExcludeContentLength;

/// Imitation of Gobuster/ffuf in Rust.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Uses directory/file enumeration mode
    Dir {
        /// The target URL
        #[arg(short, long)]
        url: Url,

        /// Path to the wordlist.
        #[arg(short, long)]
        wordlist: std::path::PathBuf,

        /// Status code that will be ignored, e.g. 404,500
        #[arg(short, long, value_delimiter = ',', default_value = "404")]
        blacklist_status_codes: Vec<StatusCode>,

        /// Content lengths that will be ignored, e.g. 20,300, or a range, e.g. 20-300
        #[arg(long, default_value_t = ExcludeContentLength::Empty)]
        exclude_length: ExcludeContentLength,

        /// Custom headers; use the format "Header1: Content1, Header2: Content2"
        #[arg(long, value_delimiter = ',', value_parser = parse_headers, required = false)]
        headers: Vec<(HeaderName, HeaderValue)>,
    }
}

fn parse_headers(s: &str) -> Result<(HeaderName, HeaderValue), Box<dyn Error + Send + Sync + 'static>> {
    let pos = s
        .find(':')
        .ok_or_else(|| format!("invalid content for `{s}`: format 'Header1: Content1, Header2: Content2'"))?;
    Ok((s[..pos].trim().parse()?, s[pos + 1..].trim().parse()?))
}


#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderName, HeaderValue};

    use crate::cli::parse_headers;

    #[test]
    fn parse_key_val_parses_colon() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let (key, val) = parse_headers("user-agent: rustbuster")?;

        assert_eq!(key, HeaderName::from_static("user-agent"));
        assert_eq!(val, HeaderValue::from_static("rustbuster"));
        Ok(())
    }

    #[test]
    #[should_panic]
    fn parse_headers_invalid_header_name() {
        parse_headers("User Agent: hello").unwrap();
    }
}
