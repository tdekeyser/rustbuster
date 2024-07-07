use clap::Parser;

pub use self::error::{Error, Result};

mod cli;
mod fuzz;
mod filters;
mod words;
mod probe;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Cli::parse();

    let mut wordlist = words::Wordlist::try_from(args.wordlist)?;
    wordlist.set_extensions(args.extensions);

    let http_probe = probe::HttpProbe::builder()
        .with_url(args.url)
        .with_method(args.method)
        .with_headers(args.headers)
        .build()?;

    let filters = filters::ProbeResponseFilters::new(
        args.filter_status_codes,
        args.filter_content_length,
        args.filter_body,
    );

    let fuzzer = fuzz::HttpFuzzer::new(
        http_probe,
        filters,
        args.delay,
        args.threads,
        args.verbose,
    );

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
