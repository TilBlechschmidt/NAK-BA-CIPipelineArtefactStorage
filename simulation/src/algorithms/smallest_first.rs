use async_trait::async_trait;
use bytesize::ByteSize;

use crate::implementation::{CleanupAttemptAlgorithm, CleanupDataSource, PipelineID};

#[derive(Debug)]
pub struct SmallestFirstAlgorithm {}

#[async_trait]
impl CleanupAttemptAlgorithm for SmallestFirstAlgorithm {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> Option<PipelineID> {
        let mut largest: Option<(PipelineID, ByteSize)> = None;

        for id in data_source.pipeline_ids() {
            let pipeline_size = data_source.pipeline_size(*id).await.unwrap();
            if let Some((_, current_size)) = largest {
                if current_size > pipeline_size {
                    largest = Some((*id, pipeline_size));
                }
            } else {
                largest = Some((*id, pipeline_size));
            }
        }

        largest.map(|l| l.0)
    }
}
