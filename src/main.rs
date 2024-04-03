use clap::Parser;

use crate::cli::{Cli, Command};

mod cli;
mod dir;
mod progress_bar;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    match &args.command {
        Some(Command::Dir {
                 url,
                 wordlist,
                 blacklist_status_codes
             }) => {
            dir::run_command(url, wordlist, blacklist_status_codes).await.ok();
        }
        None => ()
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
