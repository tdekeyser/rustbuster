use clap::Parser;

use crate::cli::{Cli, Command};
use crate::fuzz::{HttpFuzzer, FuzzError};

mod cli;
mod progress_bar;
mod exclude_length;
mod fuzz;

#[tokio::main]
async fn main() -> Result<(), FuzzError> {
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
            let fuzzer = HttpFuzzer::builder()
                .with_method(method.clone())
                .with_headers(headers.clone())?
                .with_status_code_blacklist(blacklist_status_codes.clone())
                .with_exclude_length(exclude_length.clone())
                .build()?;
            fuzzer.brute_force(url, wordlist).await?;
            Ok(())
        }
        None => Err(FuzzError("no matching command".to_string()))
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
