pub type PipelineID = i64;
pub type AccessLogEntryID = i64;
pub type MergeRequestEventID = i64;

mod algorithm;
mod algorithm_data_source;
mod data_source;
mod ml_generator;
mod simulation;
mod size_sampler;
mod state;
mod statistics;
mod static_ml_generator;

pub use algorithm::{CleanupAlgorithm, CleanupAttemptAlgorithm, FallbackCleanupAlgorithm};
pub use algorithm_data_source::CleanupDataSource;
pub use data_source::{PipelineStatus, DataSource};
pub use simulation::Simulation;
pub use statistics::{DataPoint, Statistics};
pub use ml_generator::MLGenerator;
pub use static_ml_generator::StaticMLGenerator;