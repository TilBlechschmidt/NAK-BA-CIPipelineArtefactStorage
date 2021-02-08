use async_trait::async_trait;

use crate::implementation::{CleanupDataSource, PipelineID};

use super::{Score, ScoringAlgorithm};

pub struct MergedAlgorithm {
    score: Score,
}

impl MergedAlgorithm {
    pub fn new(score: Score) -> Self {
        Self { score }
    }
}

impl Default for MergedAlgorithm {
    fn default() -> Self {
        Self { score: 5 }
    }
}

#[async_trait]
impl ScoringAlgorithm for MergedAlgorithm {
    async fn score_pipelines<'a>(
        &self,
        data_source: &CleanupDataSource<'a>,
        pipelines: &Vec<PipelineID>,
    ) -> Vec<Score> {
        let mut scores = Vec::with_capacity(pipelines.len());
        let merges = data_source.merges();

        for pipeline in pipelines {
            scores.push(if merges.contains(pipeline) {
                self.score
            } else {
                0
            });
        }

        scores
    }
}
