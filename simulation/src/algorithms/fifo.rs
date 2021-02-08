use async_trait::async_trait;

use crate::implementation::{CleanupAlgorithm, CleanupDataSource, PipelineID};

pub struct FIFOAlgorithm {}

#[async_trait]
impl CleanupAlgorithm for FIFOAlgorithm {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> PipelineID {
        *data_source.pipeline_ids().first().unwrap()
    }
}
