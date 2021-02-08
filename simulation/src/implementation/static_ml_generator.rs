use std::convert::TryInto;

use super::data_source::DataSource;
use anyhow::Result;
use async_std::{
    fs::File,
    io::{prelude::WriteExt, BufWriter},
};
use bytesize::ByteSize;
use futures::TryStreamExt;
use indicatif::{ProgressBar, ProgressStyle};

struct MLDataPoint {
    status: String,
    size: ByteSize,
    duration: i64,
    merge_after: i64,
    access_count: usize,
    no_longer_needed_after: i64,
}

impl MLDataPoint {
    fn csv_header() -> &'static str {
        "status,size,duration,merge_after,access_count,no_longer_needed_after\n"
    }

    fn serialize(&self) -> String {
        // Note that we output the size as MB instead of Bytes since CreateML apparently uses 32-Bit numbers ...
        format!(
            "{},{},{},{},{},{}\n",
            self.status,
            self.size.as_u64() / 1024 / 1024,
            self.duration,
            self.merge_after,
            self.access_count,
            self.no_longer_needed_after,
        )
    }
}

pub struct StaticMLGenerator {
    data_source: DataSource,
    progress_bar: ProgressBar,
}

impl StaticMLGenerator {
    pub async fn new(database: &str, seed: u64) -> Result<Self> {
        let data_source = DataSource::open(database, seed).await?;
        let event_count = data_source
            .all_pipelines()
            .try_collect::<Vec<_>>()
            .await?
            .len();
        let progress_bar = ProgressBar::new(event_count.try_into().unwrap());

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{prefix:>20} {msg} [{wide_bar}] {percent}% {eta} - {pos:>5} / {len:5}")
                .progress_chars("=>-"),
        );
        progress_bar.set_prefix("Generating ML data");

        Ok(Self {
            data_source,
            progress_bar,
        })
    }

    pub async fn generate(&self) -> Result<()> {
        let mut pipelines = self.data_source.all_pipelines();

        let mut f =
            BufWriter::new(File::create("../data/out/simulation/static-ml-data.csv").await?);

        f.write(MLDataPoint::csv_header().as_bytes()).await?;

        while let Some(pipeline) = pipelines.try_next().await? {
            let accesses = self
                .data_source
                .accesses_after_timestamp(pipeline.id, pipeline.created_at)
                .await?;

            let data_point = MLDataPoint {
                status: pipeline.raw_status,
                size: self.data_source.size_of_pipeline(pipeline.id).await?,
                duration: pipeline.duration,
                merge_after: 0,
                access_count: accesses.len(),
                no_longer_needed_after: accesses.first().map(|i| *i).unwrap_or(0),
            };

            f.write(data_point.serialize().as_bytes()).await?;

            self.progress_bar.inc(1);
        }

        self.progress_bar.finish();

        println!("Collected {} data points", self.progress_bar.position());

        Ok(())
    }
}
