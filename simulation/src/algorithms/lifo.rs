use async_trait::async_trait;

use crate::implementation::{CleanupAlgorithm, CleanupDataSource, PipelineID};

pub struct LIFOAlgorithm {}

#[async_trait]
impl CleanupAlgorithm for LIFOAlgorithm {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> PipelineID {
        *data_source.pipeline_ids().last().unwrap()
    }
}
