use async_trait::async_trait;

use crate::implementation::{CleanupAttemptAlgorithm, CleanupDataSource, PipelineID};

#[derive(Debug)]
pub struct MRUAlgorithm {}

#[async_trait]
impl CleanupAttemptAlgorithm for MRUAlgorithm {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> Option<PipelineID> {
        let pipelines = data_source.pipeline_ids();

        let last_accesses = pipelines.iter().filter_map(|id| {
            data_source
                .accesses(id)
                .map(|a| a.last())
                .flatten()
                .map(|l| (id, l))
        });
        let most_recently_used = last_accesses.max_by(|x, y| x.1.cmp(y.1));

        most_recently_used.map(|(id, _)| *id)
    }
}
