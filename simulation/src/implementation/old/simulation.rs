#![allow(non_snake_case)]
use std::convert::TryInto;
 
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use bytesize::ByteSize;
use chrono::{DateTime, Duration, Utc};
use futures::TryStreamExt;
use sqlx::SqliteConnection;

use super::{state::Pipeline, CleanupDataSource, PipelineID, SimulationState};

#[async_trait(?Send)]
pub trait CleanupAlgorithm {
    async fn select_pipeline<'a>(&self, data_source: &mut CleanupDataSource<'a>) -> PipelineID;
}

#[derive(sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
struct DatabasePipeline {
    id: i64,
    jobs: String,
    finished_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    duration: i64,
}

#[derive(sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
struct AccessLogEntry {
    pipeline: i64,
    timestamp: DateTime<Utc>,
}

pub struct Simulation<A: CleanupAlgorithm> {
    algorithm: A,
}

impl<A: CleanupAlgorithm> Simulation<A> {
    pub fn new(algorithm: A) -> Self {
        Self { algorithm }
    }

    async fn run_pipeline(
        &self,
        state: &mut SimulationState,
        pipeline: DatabasePipeline,
        con: &mut SqliteConnection,
    ) -> Result<()> {
        let mut pipeline_size: i64 = 0;
        for job in pipeline.jobs.split(";") {
            let sample = state.sampler.sample(job, con).await;
            // TODO Somehow handle or at least log when defaulting here, if the ratio grows too big results become less and less realistic
            pipeline_size += sample.unwrap_or_else(|_| 0);
        }

        state.add(Pipeline::new(
            pipeline.id,
            ByteSize::b(pipeline_size.try_into().unwrap()),
        ));

        if let Some(current_time) = pipeline.created_at {
            // TODO Figure out something better here. Ideally a pipeline should be available from its start not end but iterating through those is going to be a pain. Maybe store the createdAt too?
            state.current_time = current_time; // - Duration::minutes(90);
        }

        Ok(())
    }

    async fn run_next_pipeline(
        &self,
        state: &mut SimulationState,
        con: &mut SqliteConnection,
    ) -> Result<()> {
        let pipeline = sqlx::query_as::<_, DatabasePipeline>(
            "SELECT * FROM Pipeline WHERE status IS NOT NULL ORDER BY finishedAt LIMIT 1 OFFSET $1",
        )
        .bind(state.offset)
        .fetch_one(&mut *con)
        .await?;

        self.run_pipeline(state, pipeline, con).await?;

        Ok(())
    }

    async fn run_cleanup_algorithm(
        &self,
        state: &mut SimulationState,
        con: &mut SqliteConnection,
    ) -> Result<()> {
        let mut i = 0;
        while state.is_over_limit() {
            let mut data_source = CleanupDataSource::new(state, con);

            let pipeline_to_purge = self.algorithm.select_pipeline(&mut data_source).await;
            state.remove(&pipeline_to_purge);

            if i > 10_000 {
                bail!(
                    "Algorithm did not manage to get below limit after {} iterations",
                    i
                );
            }
            i += 1;
        }

        Ok(())
    }

    async fn calculate_number_of_missed_accesses(
        &self,
        state: &mut SimulationState,
        con: &mut SqliteConnection,
        previous_time: DateTime<Utc>,
    ) -> Result<()> {
        // TODO Ignore pipelines which we never processed (should be possible by adding join with WHERE status IS NOT NULL)! EDIT: Apparently that is not sufficient, we are still missing 230 pipelines
        //      By evaluating some samples it seems like the relevant pipelines are accessed before they are executed. Probably timezones ......... fuck them
        let mut accesses = sqlx::query_as::<_, AccessLogEntry>("SELECT * FROM AccessLog INNER JOIN pipeline ON pipeline.id=AccessLog.pipeline WHERE pipeline.status IS NOT NULL AND NOT isAutomatic AND NOT isIrrelevant AND CAST(strftime('%s', $1) AS integer) <= CAST(strftime('%s', timestamp) AS integer) AND CAST(strftime('%s', timestamp) AS integer) <= CAST(strftime('%s', $2) AS integer)")
        // let mut accesses = sqlx::query_as::<_, AccessLogEntry>("SELECT * FROM AccessLog WHERE NOT isAutomatic AND NOT isIrrelevant AND pipeline IS NOT NULL AND CAST(strftime('%s', $1) AS integer) <= CAST(strftime('%s', timestamp) AS integer) AND CAST(strftime('%s', timestamp) AS integer) <= CAST(strftime('%s', $2) AS integer)")
            .bind(previous_time)
            .bind(state.current_time)
            .fetch(con);

        while let Some(access) = accesses.try_next().await? {
            state.access_count += 1;

            if let Some(pipeline) = state.pipelines.get_mut(&access.pipeline) {
                pipeline.accesses.push(access.timestamp);
            } else {
                state.access_count_missed += 1;
            }
        }

        Ok(())
    }

    pub async fn advance(
        &self,
        state: &mut SimulationState,
        con: &mut SqliteConnection,
    ) -> Result<()> {
        let previous_time = state.current_time;

        self.run_next_pipeline(state, con)
            .await
            .map_err(|_| anyhow!("Simulation reached end of input"))?;

        // self.run_cleanup_algorithm(state, con).await?;

        self.calculate_number_of_missed_accesses(state, con, previous_time)
            .await?;

        Ok(())
    }
}
