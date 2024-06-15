use std::error::Error;

use clap::Parser;

use crate::cli::Cli;
use crate::words::Wordlist;

mod cli;
mod fuzz;
mod words;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let mut wordlist = Wordlist::try_from(args.wordlist)?;
    wordlist.set_extensions(args.extensions);

    let fuzzer = fuzz::HttpFuzzer::builder()
        .with_url(args.url)
        .with_method(args.method)
        .with_headers(args.headers)
        .with_delay(args.delay)
        .with_status_codes_filter(args.filter_status_codes)
        .with_content_length_filter(args.filter_content_length)
        .with_body_filter(args.filter_body)
        .with_verbose(args.verbose)
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
