use std::collections::HashMap;

use super::{
    data_source::DataSource, state::SimulationState, CleanupAlgorithm, CleanupDataSource,
    PipelineID, PipelineStatus,
};
use anyhow::{anyhow, Result};
use async_std::prelude::*;
use async_std::{fs::File, io::BufWriter};
use async_trait::async_trait;
use bytesize::ByteSize;
use futures::TryStreamExt;
use indicatif::{ProgressBar, ProgressStyle};

struct DummyAlgorithm {}

#[async_trait]
impl CleanupAlgorithm for DummyAlgorithm {
    async fn select_pipeline<'a>(&self, _: &CleanupDataSource<'a>) -> PipelineID {
        unreachable!()
    }
}

struct MLDataPoint {
    status: PipelineStatus,
    size: ByteSize,
    merged: bool,
    age: i64,
    access_count: usize,
    still_needed: bool,
    // TODO Add type/some abstraction of ref (so we can catch premaster and stuff)
}

impl MLDataPoint {
    async fn new(
        pipeline_id: i64,
        timestamp: i64,
        data_source: &DataSource,
        state: &SimulationState,
        future_access_cache: &mut HashMap<PipelineID, Vec<i64>>,
    ) -> Result<Self> {
        // Do some smart caching to not wait hours
        if !future_access_cache.contains_key(&pipeline_id) {
            let accesses = data_source
                .accesses_after_timestamp(pipeline_id, timestamp)
                .await?;
            future_access_cache.insert(pipeline_id, accesses);
        } else {
            // Drop all accesses that are in the past
            let accesses = future_access_cache.get_mut(&pipeline_id).unwrap();
            while let Some(last_event) = accesses.pop() {
                if last_event >= timestamp {
                    accesses.push(last_event);
                    // println!("Reduced {} down to {} accesses!", pipeline_id, accesses.len());
                    break;
                }
            }
        }

        // Collect all the properties
        let status = data_source.status_of_pipeline(pipeline_id).await?;
        let size = data_source.size_of_pipeline(pipeline_id).await?;
        // let still_needed = data_source
        //     .will_pipeline_be_accessed_after_timestamp(pipeline_id, timestamp)
        //     .await?;
        let still_needed = future_access_cache
            .get(&pipeline_id)
            .map_or(false, |a| a.len() > 0);
        let merged = state.merges.contains(&pipeline_id);

        let age = timestamp
            - state
                .storage_times
                .get(&pipeline_id)
                .ok_or(anyhow!("No storage time for pipline!"))?;

        let access_count = state.accesses.get(&pipeline_id).map_or(0, |a| a.len());

        Ok(Self {
            status,
            size,
            merged,
            age,
            access_count,
            still_needed,
        })
    }

    fn csv_header() -> &'static str {
        "status,size,merged,age,accessCount,stillNeeded\n"
    }

    fn serialize(&self) -> String {
        format!(
            "{:?},{},{},{},{},{}\n",
            self.status,
            self.size.as_u64(),
            self.merged as u8,
            self.age,
            self.access_count,
            self.still_needed as u8
        )
    }
}

pub struct MLGenerator {
    data_source: DataSource,
    progress_bar: ProgressBar,
}

impl MLGenerator {
    pub async fn prepare(database: &str, seed: u64) -> Result<Self> {
        let data_source = DataSource::open(database, seed).await?;
        let event_count = data_source.event_count().await?;
        let progress_bar = ProgressBar::new(event_count);

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
        self.progress_bar.set_position(0);

        // Since we set the limit to some cosmic number which will only be reached in a fault state
        // the algorithm will never be called. Thus we can just pass a DummyAlgorithm which does nothing and unwinds if it is called.
        let limit = ByteSize::pb(9000);
        let algorithm = Box::new(DummyAlgorithm {});

        let mut state = SimulationState::new(&self.data_source, algorithm, limit);
        let mut event_stream = self.data_source.events();

        let mut future_access_cache: HashMap<PipelineID, Vec<i64>> = HashMap::new();

        let mut f =
            BufWriter::new(File::create("../data/out/simulation/ml-data-reduced.csv").await?);

        f.write(MLDataPoint::csv_header().as_bytes()).await?;

        let mut i = 0;
        let mut generated_count = 0;
        while let Some(event) = event_stream.try_next().await? {
            state.process(event).await?;

            let data_points = self
                .generate_data_for_state(&state, &mut future_access_cache)
                .await?;

            generated_count += data_points.len();

            for data_point in data_points {
                f.write(data_point.serialize().as_bytes()).await?;
            }

            // Remove everything older than 5 days to reduce the data amount
            state.remove_pipelines_older_than(60 * 60 * 24 * 3).await?;
            // println!("{}", state.stored_pipelines.len());

            if i % 100 == 0 {
                self.progress_bar.inc(100);
            }

            i += 1;
        }

        self.progress_bar.finish();

        println!("Collected {} data points", generated_count);

        Ok(())
    }

    async fn generate_data_for_state(
        &self,
        state: &SimulationState,
        future_access_cache: &mut HashMap<PipelineID, Vec<i64>>,
    ) -> Result<Vec<MLDataPoint>> {
        let timestamp = state
            .latest_event
            .ok_or(anyhow!("Simulation has no latest event"))?
            .timestamp;

        let mut data_points = Vec::with_capacity(state.stored_pipelines.len());

        for pipeline_id in state.stored_pipelines.iter() {
            // TODO Skip pipelines which are created but not finished
            data_points.push(
                MLDataPoint::new(
                    *pipeline_id,
                    timestamp,
                    &self.data_source,
                    &state,
                    future_access_cache,
                )
                .await?,
            );
        }

        Ok(data_points)
    }
}
