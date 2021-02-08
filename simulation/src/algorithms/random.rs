use async_std::sync::Mutex;
use async_trait::async_trait;
use rand::{prelude::StdRng, Rng, SeedableRng};

use crate::implementation::{CleanupAlgorithm, CleanupDataSource, PipelineID};

pub struct RandomAlgorithm {
    rng: Mutex<StdRng>,
}

impl RandomAlgorithm {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
        }
    }
}

#[async_trait]
impl CleanupAlgorithm for RandomAlgorithm {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> PipelineID {
        let ids = data_source.pipeline_ids();
        let index = self.rng.lock().await.gen_range(0..ids.len());

        // TODO This can be done more efficiently by skipping
        *ids.iter().collect::<Vec<&PipelineID>>()[index]
    }
}
