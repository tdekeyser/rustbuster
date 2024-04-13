use std::fs;
use std::path::PathBuf;

use clap::Parser;
use url::Url;

use crate::cli::{Cli, Command};
use crate::http::{HttpClient, HttpError};

mod cli;
mod progress_bar;
mod exclude_length;
mod http;

#[tokio::main]
async fn main() -> Result<(), HttpError> {
    let args = Cli::parse();
    match &args.command {
        Some(Command::Dir {
                 url,
                 wordlist,
                 blacklist_status_codes,
                 exclude_length
             }) => {
            let http_client = HttpClient::builder()
                .with_status_code_blacklist(blacklist_status_codes.clone())
                .with_exclude_length(exclude_length.clone())
                .build()?;
            run_dir_command(url, wordlist, http_client).await?;
            Ok(())
        }
        None => Err(HttpError("no matching command".to_string()))
    }
}


async fn run_dir_command(url: &Url,
                         wordlist: &PathBuf,
                         http_client: HttpClient) -> Result<(), HttpError> {
    let wordlist = fs::read_to_string(wordlist).expect("file not found");
    let pb = progress_bar::new(wordlist.lines().count() as u64);

    for word in wordlist.lines() {
        let request_url = format!("{}{}", url, word);

        match http_client.probe(request_url).await? {
            Some(response) => pb.println(format!(
                "/{:<30} ({:>10}) [Size: {:?}]",
                word, response.status_code, response.content_length)
            ),
            None => ()
        }

        pb.inc(1);
    }
    Ok(())
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
