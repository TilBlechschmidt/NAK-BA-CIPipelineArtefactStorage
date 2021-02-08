use anyhow::Result;
use bytesize::ByteSize;
use futures::TryStreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use super::{data_source::DataSource, state::SimulationState, CleanupAlgorithm, Statistics};

pub struct Simulation {
    statistics: Statistics,
    data_source: DataSource,
    progress_bar: ProgressBar,
}

impl Simulation {
    // pub async fn new(database: &str, seed: u64, progress: &MultiProgress) -> Result<Self> {
    //     let data_source = DataSource::open(database, seed).await?;
    //     let event_count = data_source.event_count().await?;
    //     let progress_bar = progress.add(ProgressBar::new(event_count));

    //     progress_bar.set_style(
    //         ProgressStyle::default_bar()
    //             .template("{prefix:>20} {msg} [{wide_bar}] {percent}% {eta} - {pos:>5} / {len:5}")
    //             .progress_chars("=>-"),
    //     );

    //     data_source.populate_size_samples().await?;

    //     Ok(Self {
    //         statistics: Statistics::new(),
    //         data_source,
    //         progress_bar,
    //     })
    // }

    pub async fn prepare(data_source: DataSource, progress: &MultiProgress) -> Result<Self> {
        let event_count = data_source.event_count().await?;
        let progress_bar = progress.add(ProgressBar::new(event_count));

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{prefix:>20} {msg} [{wide_bar}] {percent}% {eta} - {pos:>5} / {len:5}")
                .progress_chars("=>-"),
        );

        Ok(Self {
            statistics: Statistics::new(),
            data_source,
            progress_bar,
        })
    }

    pub fn set_name(&self, name: &str) {
        self.progress_bar.set_prefix(name);
    }

    pub async fn run(
        mut self,
        algorithm: Box<dyn CleanupAlgorithm>,
        storage_limit: ByteSize,
    ) -> Result<Statistics> {
        let mut state = SimulationState::new(&self.data_source, algorithm, storage_limit);
        let mut event_stream = self.data_source.events();

        let mut i = 0;
        while let Some(event) = event_stream.try_next().await? {
            state.process(event).await?;
            state.cleanup().await?;
            self.statistics.record(&state);

            // Don't waste immense resources on terminal I/O
            i += 1;
            if i > 250 {
                self.progress_bar.inc(i);
                self.progress_bar.set_message(&format!(
                    "{:>5.2}% missed",
                    self.statistics.current_miss_percentage() * 100.0
                ));
                i = 0;
            }
        }

        self.progress_bar.finish();

        Ok(self.statistics)
    }
}
