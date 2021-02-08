use async_trait::async_trait;

use crate::implementation::{
    CleanupAttemptAlgorithm, CleanupDataSource, PipelineID, PipelineStatus,
};

#[derive(Debug)]
pub struct StatusAlgorithm {}

#[async_trait]
impl CleanupAttemptAlgorithm for StatusAlgorithm {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> Option<PipelineID> {
        // Deletes the oldest successful pipeline
        for pipeline in data_source.pipeline_ids().iter() {
            if let Ok(status) = data_source.pipeline_status(*pipeline).await {
                if status == PipelineStatus::Success {
                    return Some(*pipeline);
                }
            }
        }

        // Fallback
        // Deletes the oldest pipeline which did anything but fail
        for pipeline in data_source.pipeline_ids().iter() {
            if let Ok(status) = data_source.pipeline_status(*pipeline).await {
                if status != PipelineStatus::Failed {
                    return Some(*pipeline);
                }
            }
        }

        None
    }
}
