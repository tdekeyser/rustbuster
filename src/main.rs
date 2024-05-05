use std::error::Error;
use clap::Parser;

use crate::cli::{Cli, Command};

mod cli;
mod progress_bar;
mod exclude_length;
mod fuzz;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    match &args.command {
        Some(Command::Dir {
                 url,
                 wordlist,
                 method,
                 headers,
                 blacklist_status_codes,
                 exclude_length
             }) => {
            let fuzzer = fuzz::HttpFuzzer::builder()
                .with_method(method.clone())
                .with_headers(headers.clone())
                .with_status_code_blacklist(blacklist_status_codes.clone())
                .with_exclude_length(exclude_length.clone())
                .build()?;
            fuzzer.brute_force(url, wordlist).await?;
            Ok(())
        }
        None => Err("no matching command".into())
    }
}


#[cfg(test)]
mod tests {
    use crate::cli::Cli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
