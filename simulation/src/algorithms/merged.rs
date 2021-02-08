use async_trait::async_trait;

use crate::implementation::{CleanupAttemptAlgorithm, CleanupDataSource, PipelineID};

#[derive(Debug)]
pub struct BranchMergedAlgorithm {}

#[async_trait]
impl CleanupAttemptAlgorithm for BranchMergedAlgorithm {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> Option<PipelineID> {
        data_source.merges().first().map(|id| *id)
        
        
        // let ids: Vec<PipelineID> = data_source.pipelines_ids().map(|id| *id).collect();

        // for id in ids.iter() {
        //     if let Ok(Some(event)) = data_source
        //         .merge_request_events(*id)
        //         .await
        //         .map(|mut events| events.pop())
        //     {
        //         if event.status == "merged" {
        //             return *id;
        //         }
        //     }
        // }

        // eprintln!("No merged pipelines found :(");
        // self.fallback.select_pipeline(data_source).await
    }
}
