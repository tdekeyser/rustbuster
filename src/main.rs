use clap::{Parser, Subcommand};
use url::Url;

mod dir;
mod progress_bar;

/// Imitation of Gobuster/ffuf in Rust.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Uses directory/file enumeration mode
    Dir {
        /// The target URL
        #[arg(short, long)]
        url: Url,

        /// Path to the wordlist.
        #[arg(short, long)]
        wordlist: std::path::PathBuf,
    }
}


#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match &args.command {
        Some(Commands::Dir { url, wordlist }) => {
            dir::run_command(url, wordlist).await.ok();
        }
        None => ()
    }
}


#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}