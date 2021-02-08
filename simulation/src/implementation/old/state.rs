use std::{collections::BTreeMap, convert::TryInto};

use super::{size_sampler::JobSizeSampler, PipelineID};
use anyhow::Result;
use bytesize::ByteSize;
use chrono::{DateTime, NaiveDateTime, Utc};

pub struct Pipeline {
    pub id: PipelineID,
    pub size: ByteSize,
    pub accesses: Vec<DateTime<Utc>>,
}

impl Pipeline {
    pub fn new(id: PipelineID, size: ByteSize) -> Self {
        Self {
            id,
            size,
            accesses: Vec::new(),
        }
    }
}

pub struct SimulationState {
    pub sampler: JobSizeSampler,
    pub offset: i64,

    pub limit: ByteSize,
    pub bytes: u64,

    pub pipelines: BTreeMap<PipelineID, Pipeline>,

    pub current_time: DateTime<Utc>,

    pub access_count: u32,
    pub access_count_missed: u32,
}

impl SimulationState {
    pub async fn new(seed: u64, limit: ByteSize) -> Result<Self> {
        let sampler = JobSizeSampler::new(seed);

        Ok(Self {
            sampler,
            offset: 0,

            limit,
            bytes: 0,

            pipelines: BTreeMap::new(),

            // 1970-1-1
            current_time: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
            access_count: 0,
            access_count_missed: 0,
        })
    }

    pub fn add(&mut self, pipeline: Pipeline) {
        self.offset += 1;
        self.bytes += pipeline.size.as_u64();
        self.pipelines.insert(pipeline.id, pipeline);
    }

    pub fn remove(&mut self, id: &PipelineID) {
        if let Some(entry) = self.pipelines.remove(id) {
            self.bytes -= entry.size.as_u64();
        }
    }

    pub fn is_over_limit(&self) -> bool {
        ByteSize::b(self.bytes) > self.limit
    }

    pub fn deleted_pipeline_count(&self) -> usize {
        let offset: usize = self.offset.try_into().unwrap();
        offset - self.pipelines.len()
    }

    pub fn missed_percentage(&self) -> f64 {
        let missed: f64 = self.access_count_missed.into();
        let total: f64 = self.access_count.into();

        missed / total
    }
}
