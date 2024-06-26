use std::time::Duration;

use tokio::time;

use crate::filters::ProbeResponseFilters;
use crate::probe::HttpProbe;
use crate::Result;
use crate::words::Wordlist;

mod progress_bar;

pub struct HttpFuzzer {
    http_probe: HttpProbe,
    filters: ProbeResponseFilters,
    delay: Option<u64>,
    verbose: bool,
}

impl HttpFuzzer {
    pub fn new(http_probe: HttpProbe,
               filters: ProbeResponseFilters,
               delay: f32,
               verbose: bool) -> Self {
        let delay = match delay {
            0.0 => None,
            _ => Some((delay * 1000.0) as u64)
        };
        Self { http_probe, filters, delay, verbose }
    }

    pub async fn brute_force(&self, wordlist: Wordlist) -> Result<()> {
        let pb = progress_bar::new(wordlist.len() as u64);

        for word in wordlist.iter() {
            pb.inc(1);

            let r = self.http_probe.probe(&word).await?;

            if let Some(response) = self.filters.filter(r) {
                pb.suspend(|| println!("{}", response.display(self.verbose)))
            }

            match self.delay {
                Some(delay) => time::sleep(Duration::from_millis(delay)).await,
                None => ()
            }
        }

        Ok(())
    }
}
