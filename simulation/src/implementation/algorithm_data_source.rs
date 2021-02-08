use super::{
    data_source::{DataSource, PipelineStatus},
    state::SimulationState,
    PipelineID,
};
use anyhow::Result;
use bytesize::ByteSize;
use std::collections::BTreeSet;

pub struct CleanupDataSource<'a> {
    state: &'a SimulationState,
    data_source: &'a DataSource,
}

impl<'a> CleanupDataSource<'a> {
    pub fn new(state: &'a SimulationState, data_source: &'a DataSource) -> Self {
        Self { state, data_source }
    }

    /// List of stored pipeline IDs ordered by insertion time.
    /// Last item in slice equals latest insertion.
    pub fn pipeline_ids(&self) -> &BTreeSet<PipelineID> {
        &self.state.stored_pipelines
    }

    pub async fn pipeline_size(&self, id: PipelineID) -> Result<ByteSize> {
        Ok(self.data_source.size_of_pipeline(id).await?)
    }

    pub async fn pipeline_status(&self, id: PipelineID) -> Result<PipelineStatus> {
        Ok(self.data_source.status_of_pipeline(id).await?)
    }

    pub fn pipeline_age(&self, id: PipelineID) -> Option<i64> {
        let current_time = self.state.latest_event.map(|e| e.timestamp).unwrap_or(0);
        let storage_time = self.state.storage_times.get(&id);

        storage_time.map(|t| current_time - t)
    }

    pub fn accesses(&self, id: &PipelineID) -> Option<&Vec<i64>> {
        self.state.accesses.get(id)
    }

    pub fn merges(&self) -> &BTreeSet<PipelineID> {
        &self.state.merges
    }
}
