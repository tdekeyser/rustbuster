use clap::{Parser, Subcommand};
use reqwest::StatusCode;
use url::Url;

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
    }
}