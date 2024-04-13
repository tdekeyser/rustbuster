use clap::{Parser, Subcommand};
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
    }
}
