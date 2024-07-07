use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;
use tokio::time;

use crate::filters::ProbeResponseFilters;
use crate::probe::HttpProbe;
use crate::Result;
use crate::words::Wordlist;

mod progress_bar;

pub struct HttpFuzzer {
    http_probe: HttpProbe,
    filters: ProbeResponseFilters,
    delay: Option<Duration>,
    num_threads: usize,
    verbose: bool,
}

impl HttpFuzzer {
    pub fn new(http_probe: HttpProbe,
               filters: ProbeResponseFilters,
               delay: f32,
               num_threads: usize,
               verbose: bool) -> Self {
        let delay = if delay != 0.0 { Some(Duration::from_secs_f32(delay)) } else { None };
        Self { http_probe, filters, delay, num_threads, verbose }
    }

    pub async fn brute_force(&self, wordlist: Wordlist) -> Result<()> {
        let pb = Arc::new(progress_bar::new(wordlist.len() as u64));
        let semaphore = Arc::new(Semaphore::new(self.num_threads));

        let mut tasks = Vec::new();

        for word in wordlist.iter() {
            let pb = Arc::clone(&pb);
            let semaphore = Arc::clone(&semaphore);

            let http_probe = self.http_probe.clone();
            let filters = self.filters.clone();
            let verbose = self.verbose;
            let delay = self.delay;

            let task = tokio::spawn(async move {
                HttpFuzzer::process_word(word, http_probe, filters, verbose, delay, pb, semaphore).await
            });

            tasks.push(task);
        }

        for task in tasks {
            task.await??;
        }

        Ok(())
    }

    async fn process_word(
        word: String,
        http_probe: HttpProbe,
        filters: ProbeResponseFilters,
        verbose: bool,
        delay: Option<Duration>,
        pb: Arc<indicatif::ProgressBar>,
        semaphore: Arc<Semaphore>,
    ) -> Result<()> {
        let _permit = semaphore.acquire().await;
        pb.inc(1);

        let r = http_probe.probe(&word).await?;

        if let Some(response) = filters.filter(r) {
            pb.suspend(|| println!("{}", response.display(verbose)));
        }

        if let Some(d) = delay {
            time::sleep(d).await;
        }

        Ok(())
    }
}
