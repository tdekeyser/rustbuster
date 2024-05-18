use std::error::Error;

use clap::Parser;

use crate::cli::Cli;
use crate::words::Wordlist;

mod cli;
mod progress_bar;
mod fuzz;
mod words;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let wordlist = Wordlist::try_from(args.wordlist)?
        .expand(args.extensions);

    let fuzzer = fuzz::HttpFuzzer::builder()
        .with_url(args.url)
        .with_method(args.method)
        .with_headers(args.headers)
        .with_status_codes_filter(args.filter_status_codes)
        .with_content_length_filter(args.filter_content_length)
        .with_body_filter(args.filter_body)
        .build()?;

    fuzzer.brute_force(wordlist).await
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
