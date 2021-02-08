use anyhow::{bail, Result};
use async_std::sync::Mutex;
use bytesize::ByteSize;
use futures::{stream::BoxStream, TryStreamExt};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{collections::HashMap, convert::TryInto, sync::Arc};

use super::{size_sampler::JobSizeSampler, AccessLogEntryID, MergeRequestEventID, PipelineID};

#[derive(Debug, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SimulationEventKind {
    PipelineCreated = 0,
    PipelineFinished = 1,
    MergeRequestEvent = 2,
    Access = 3,
}

#[derive(sqlx::FromRow, Debug, Clone, Copy)]
pub struct SimulationEvent {
    pub id: i64,
    pub timestamp: i64,
    pub kind: SimulationEventKind,
    pub key: i64,
}

#[derive(sqlx::FromRow)]
pub struct AccessLogEntry {
    pub timestamp: i64,
    pub pipeline: PipelineID,
}

#[derive(sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
pub struct MergeRequestEvent {
    pub source_branch: String,
    pub status: String, // opened, closed, merged
    pub action: String, // merge, ...
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelineStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
    Skipped,

    // Build status
    Created,
    Manual,
}

impl PipelineStatus {
    pub fn from_string(source: &str) -> Self {
        match source {
            "pending" => PipelineStatus::Pending,
            "running" => PipelineStatus::Running,
            "success" => PipelineStatus::Success,
            "failed" => PipelineStatus::Failed,
            "canceled" => PipelineStatus::Cancelled,
            "skipped" => PipelineStatus::Skipped,
            "created" => PipelineStatus::Created,
            "manual" => PipelineStatus::Manual,
            e => {
                eprintln!("Encountered unexpected pipeline status: '{}'", e);
                unreachable!()
            }
        }
    }
}

#[derive(sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
pub struct Pipeline {
    pub id: PipelineID,
    pub jobs: String,
    // pub finished_at: Option<i64>,
    pub created_at: i64,
    pub duration: i64,
    #[sqlx(rename = "status")]
    pub raw_status: String,
}

// impl Pipeline {
//     pub fn status(&self) -> PipelineStatus {
//         PipelineStatus::from_string(&self.raw_status)
//     }
// }

#[derive(Clone)]
pub struct DataSource {
    con: SqlitePool,
    sampler: Arc<Mutex<JobSizeSampler>>,

    sizes: Arc<Mutex<HashMap<PipelineID, i64>>>,

    status_cache: Arc<Mutex<HashMap<PipelineID, PipelineStatus>>>,
}

impl DataSource {
    pub async fn open(database: &str, seed: u64) -> Result<Self> {
        // Note: When running in parallel the JobSizeSampler might not be called in the same order due to thread timing
        //       This might yield indeterministic behavior!
        // TODO: Circumvent this by pre-populating the sampler.
        let con = SqlitePoolOptions::new()
            .min_connections(64)
            .max_connections(100)
            .connect(database)
            .await?;

        Ok(Self {
            // con: SqlitePool::connect(database).await?,
            con,
            sampler: Arc::new(Mutex::new(JobSizeSampler::new(seed))),
            sizes: Arc::new(Mutex::new(HashMap::new())),
            status_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn populate_size_samples(&self) -> Result<ByteSize> {
        let mut event_stream = self.events();
        let mut total_size = ByteSize::b(0);

        while let Some(event) = event_stream.try_next().await? {
            match event.kind {
                SimulationEventKind::PipelineCreated | SimulationEventKind::PipelineFinished => {
                    if let Ok(size) = self.size_of_pipeline(event.key).await {
                        if std::env::var("SIZE_POPULATION_DUMP").is_ok() {
                            println!("{}", size.as_u64());
                        }
                        total_size = total_size + size;
                    }
                }
                _ => {}
            }
        }

        Ok(total_size)
    }

    pub fn events(&self) -> BoxStream<Result<SimulationEvent, sqlx::Error>> {
        sqlx::query_as("SELECT * FROM SimulationEvent ORDER BY timestamp").fetch(&self.con)
    }

    pub async fn event_count(&self) -> Result<u64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM SimulationEvent")
            .fetch_one(&self.con)
            .await?;
        Ok(row.0.try_into()?)
    }

    pub async fn access_log_entry(&self, id: AccessLogEntryID) -> Result<AccessLogEntry> {
        Ok(
            sqlx::query_as("SELECT timestamp,pipeline FROM AccessLog WHERE id=$1")
                .bind(id)
                .fetch_one(&self.con)
                .await?,
        )
    }

    pub async fn merge_request_event(&self, id: MergeRequestEventID) -> Result<MergeRequestEvent> {
        Ok(sqlx::query_as(
            "SELECT sourceBranch,status,action FROM MergeRequestEvent WHERE eventID=$1",
        )
        .bind(id)
        .fetch_one(&self.con)
        .await?)
    }

    pub async fn pipeline(&self, id: PipelineID) -> Result<Pipeline> {
        Ok(
            sqlx::query_as("SELECT id,jobs,status,duration,createdAt FROM Pipeline WHERE id=$1")
                .bind(id)
                .fetch_one(&self.con)
                .await?,
        )
    }

    pub async fn pipelines_for_ref(&self, pipeline_ref: String) -> Result<Vec<Pipeline>> {
        Ok(
            sqlx::query_as("SELECT id,jobs,status,duration,createdAt FROM Pipeline WHERE ref=$1")
                .bind(pipeline_ref)
                .fetch_all(&self.con)
                .await?,
        )
    }

    /// Evaluates whether a pipeline has metadata from Gitlab or is derived from an AccessLogEntry (the latter happens especially in the beginning where not all pipelines are available)
    /// Also checks if a pipeline has size data available
    pub async fn pipeline_is_populated(&self, id: PipelineID) -> Result<bool> {
        let row: (Option<i64>,) = sqlx::query_as("SELECT createdAt FROM Pipeline WHERE id=$1")
            .bind(id)
            .fetch_one(&self.con)
            .await?;

        let has_gitlab_data = row.0 != None;
        let has_size = self.size_of_pipeline(id).await.is_ok();

        Ok(has_gitlab_data && has_size)
    }

    pub async fn size_of_pipeline(&self, id: PipelineID) -> Result<ByteSize> {
        let mut sizes = self.sizes.lock().await;

        if let Some(size) = sizes.get(&id) {
            return Ok(ByteSize::b((*size).try_into().unwrap()));
        }

        let mut total_size = 0;
        let mut sampler = self.sampler.lock().await;

        let mut missed_count = 0;
        for job in self.pipeline(id).await?.jobs.split(";") {
            match sampler.sample(job, &self.con).await {
                Ok(size) => total_size += size,
                Err(_e) => {
                    missed_count += 1;
                    // eprintln!("Ignoring job {} of pipeline {}: {:?}", job, id, e)
                }
            }
        }

        if missed_count > 0 {
            bail!("Not enough size samples available for pipeline {}!", id);
        }

        sizes.insert(id, total_size);

        Ok(ByteSize::b(total_size.try_into().unwrap()))
    }

    pub async fn status_of_pipeline(&self, id: PipelineID) -> Result<PipelineStatus> {
        if let Some(cache_value) = self.status_cache.lock().await.get(&id) {
            return Ok(*cache_value);
        }

        let row: (String,) = sqlx::query_as("SELECT status FROM Pipeline WHERE id=$1")
            .bind(id)
            .fetch_one(&self.con)
            .await?;

        let status = PipelineStatus::from_string(&row.0);

        self.status_cache.lock().await.insert(id, status);

        Ok(status)
    }

    #[allow(dead_code)]
    pub async fn will_pipeline_be_accessed_after_timestamp(
        &self,
        id: PipelineID,
        timestamp: i64,
    ) -> Result<bool> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM AccessLog WHERE pipeline=$1 AND timestamp>$2 AND NOT isIrrelevant AND NOT isAutomatic")
                .bind(id)
                .bind(timestamp)
                .fetch_one(&self.con)
                .await?;

        Ok(row.0 > 0)
    }

    pub async fn accesses_after_timestamp(
        &self,
        id: PipelineID,
        timestamp: i64,
    ) -> Result<Vec<i64>> {
        let rows: Vec<(i64,)> = sqlx::query_as("SELECT timestamp FROM AccessLog WHERE pipeline=$1 AND timestamp>$2 AND NOT isIrrelevant AND NOT isAutomatic ORDER BY timestamp DESC")
            .bind(id)
            .bind(timestamp)
            .fetch_all(&self.con)
            .await?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub fn all_pipelines(&self) -> BoxStream<Result<Pipeline, sqlx::Error>> {
        sqlx::query_as("SELECT * FROM Pipeline WHERE createdAt > 0").fetch(&self.con)
    }
}
