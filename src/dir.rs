use std::fs;
use std::path::PathBuf;

use reqwest::{Client, Error, StatusCode};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use url::Url;

use crate::progress_bar;

pub async fn run_command(url: &Url,
                         wordlist: &PathBuf,
                         status_code_blacklist: &Vec<StatusCode>) -> Result<(), Error> {
    let wordlist = fs::read_to_string(wordlist).expect("file not found");
    let pb = progress_bar::new(wordlist.lines().count() as u64);

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("rustbuster"));

    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    for word in wordlist.lines() {
        let request_url = format!("{}{}", url, word);
        let response = client.get(request_url)
            .send()
            .await?;

        if !status_code_blacklist.contains(&response.status()) {
            pb.println(
                format!("/{:<30} ({:>10}) [Size: {:?}]",
                        word, response.status(), response.text().await.unwrap().len()))
        }

        pb.inc(1)
    }

    Ok(())
}
