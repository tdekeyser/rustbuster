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

    let http_client = configure_http_client();

    for word in wordlist.lines() {
        let status_code = get_status_from_request(&http_client, url, word).await?;

        if !status_code_blacklist.contains(&status_code) {
            pb.println(format!("/{:<30} [{:>10}]", word, status_code))
        }

        pb.inc(1)
    }

    Ok(())
}

fn configure_http_client() -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("rustbuster"));

    Client::builder().default_headers(headers).build().expect("error building http client")
}

async fn get_status_from_request(http_client: &Client,
                                 url: &Url,
                                 word: &str) -> Result<StatusCode, Error> {
    let mut request_url = String::from(url.as_str());
    request_url.push_str(word);

    let status_code = http_client.get(request_url).send().await?.status();

    Ok(status_code)
}
