use std::f64::consts::PI;

use super::{Score, ScoringAlgorithm};
use crate::implementation::{CleanupDataSource, PipelineID};
use async_trait::async_trait;

pub struct AgeAlgorithm {
    threshold: i64,
    score: Score,
}

impl AgeAlgorithm {
    pub fn new(threshold: i64, score: Score) -> Self {
        Self { threshold, score }
    }
}

impl Default for AgeAlgorithm {
    fn default() -> Self {
        Self {
            threshold: 60 * 60 * 24 * 2,
            score: 30,
        }
    }
}

#[async_trait]
impl ScoringAlgorithm for AgeAlgorithm {
    async fn score_pipelines<'a>(
        &self,
        data_source: &CleanupDataSource<'a>,
        pipelines: &Vec<PipelineID>,
    ) -> Vec<Score> {
        let mut scores = Vec::with_capacity(pipelines.len());

        for pipeline in pipelines {
            let score = if let Some(age) = data_source.pipeline_age(*pipeline) {
                let age = age as f64;
                let threshold = self.threshold as f64;
                let percentage = 1.0f64.max(0.0f64.min(age / threshold));
                // y = -0.5 * (cos(pi * x) - 1)
                let interpolated = -0.5 * ((PI * percentage).cos() - 1.0);

                (interpolated * (self.score as f64)).round() as Score
            } else {
                0
            };

            scores.push(score);

            // scores.push(
            //     if data_source
            //         .pipeline_age(*pipeline)
            //         .map(|i| i < self.threshold)
            //         .unwrap_or(false)
            //     {
            //         self.score
            //     } else {
            //         0
            //     },
            // );
        }

        scores
    }
}
