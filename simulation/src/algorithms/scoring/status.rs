use async_trait::async_trait;

use crate::implementation::{CleanupDataSource, PipelineID, PipelineStatus};

use super::{Score, ScoringAlgorithm};

pub struct StatusAlgorithm {
    running: Score,
    success: Score,
    failed: Score,
    cancelled: Score,
}

impl StatusAlgorithm {
    pub fn new(running: Score, success: Score, failed: Score, cancelled: Score) -> Self {
        Self { running, success, failed, cancelled }
    }
}

impl Default for StatusAlgorithm {
    fn default() -> Self {
        Self {
            running: -1,
            success: 10,
            failed: 1,
            cancelled: 3,
        }
    }
}

#[async_trait]
impl ScoringAlgorithm for StatusAlgorithm {
    async fn score_pipelines<'a>(
        &self,
        data_source: &CleanupDataSource<'a>,
        pipelines: &Vec<PipelineID>,
    ) -> Vec<Score> {
        let mut scores = Vec::with_capacity(pipelines.len());

        for pipeline in pipelines {
            let status = data_source.pipeline_status(*pipeline).await.unwrap();

            let score = match status {
                PipelineStatus::Running => self.running,
                PipelineStatus::Success => self.success,
                PipelineStatus::Failed => self.failed,
                PipelineStatus::Cancelled => self.cancelled,

                _ => 0,
            };

            scores.push(score);
        }

        scores
    }
}
