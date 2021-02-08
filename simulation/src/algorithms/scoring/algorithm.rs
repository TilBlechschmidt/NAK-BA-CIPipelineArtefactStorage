use crate::implementation::{CleanupAlgorithm, CleanupDataSource, PipelineID};
use async_trait::async_trait;

pub type Score = i32;

#[async_trait]
pub trait ScoringAlgorithm: Send + Sync {
    async fn score_pipelines<'a>(
        &self,
        data_source: &CleanupDataSource<'a>,
        pipelines: &Vec<PipelineID>,
    ) -> Vec<Score>;
}

pub struct ScoringAlgorithmManager {
    algorithms: Vec<Box<dyn ScoringAlgorithm>>,
}

impl ScoringAlgorithmManager {
    pub fn new(algorithms: Vec<Box<dyn ScoringAlgorithm>>) -> Self {
        Self { algorithms }
    }
}

#[async_trait]
impl CleanupAlgorithm for ScoringAlgorithmManager {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> PipelineID {
        let pipeline_ids = data_source
            .pipeline_ids()
            .iter()
            .map(|id| *id)
            .collect::<Vec<_>>();

        // Could be done nicer with iterators, map and such but the async/await makes that annoyingly hard
        let mut scores = vec![0; pipeline_ids.len()];
        for algorithm in self.algorithms.iter() {
            let current_scores = algorithm.score_pipelines(data_source, &pipeline_ids).await;

            for i in 0..scores.len() {
                scores[i] += current_scores[i];
            }
        }

        // Find the first maximum value in the list of scores
        let mut highscore = (0, scores[0]);
        for (i, score) in scores.into_iter().skip(1).enumerate() {
            if highscore.1 < score {
                highscore = (i, score)
            }
        }

        pipeline_ids[highscore.0]
    }
}
