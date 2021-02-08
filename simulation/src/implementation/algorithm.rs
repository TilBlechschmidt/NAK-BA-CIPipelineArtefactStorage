use async_std::sync::Mutex;
use async_trait::async_trait;

use super::{CleanupDataSource, PipelineID};

#[async_trait]
pub trait CleanupAlgorithm: Send + Sync {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> PipelineID;
}

#[async_trait]
pub trait CleanupAttemptAlgorithm: std::fmt::Debug + Send + Sync {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> Option<PipelineID>;
}

pub struct FallbackCleanupAlgorithm {
    algorithms: Vec<Box<dyn CleanupAttemptAlgorithm>>,
    fallback: Box<dyn CleanupAlgorithm>,
    total_count: Mutex<usize>,
    fallback_count: Mutex<usize>,
}

impl FallbackCleanupAlgorithm {
    pub fn new(
        algorithms: Vec<Box<dyn CleanupAttemptAlgorithm>>,
        fallback: Box<dyn CleanupAlgorithm>,
    ) -> Self {
        Self {
            algorithms,
            fallback,
            total_count: Mutex::new(0),
            fallback_count: Mutex::new(0),
        }
    }
}

#[async_trait]
impl CleanupAlgorithm for FallbackCleanupAlgorithm {
    async fn select_pipeline<'a>(&self, data_source: &CleanupDataSource<'a>) -> PipelineID {
        if std::env::var("FALLBACK_DEBUG").is_ok() {
            println!("{:?} => {} / {}", self.algorithms, self.fallback_count.lock().await, self.total_count.lock().await);
        }

        *self.total_count.lock().await += 1;

        // TODO Track which algorithm answered how many requests
        for algorithm in self.algorithms.iter() {
            if let Some(selected_pipeline) = algorithm.select_pipeline(data_source).await {
                return selected_pipeline;
            }
        }

        *self.fallback_count.lock().await += 1;
        self.fallback.select_pipeline(data_source).await
    }
}
