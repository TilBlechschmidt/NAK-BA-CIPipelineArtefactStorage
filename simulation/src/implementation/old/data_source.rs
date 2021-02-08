use anyhow::Result;
use std::collections::btree_map::{Keys, Values};

use sqlx::SqliteConnection;

use super::{state::Pipeline, PipelineID, SimulationState};

#[derive(sqlx::FromRow, Debug)]
#[sqlx(rename_all = "camelCase")]
pub struct MergeRequestEvent {
    pub status: String
}

pub struct CleanupDataSource<'a> {
    state: &'a mut SimulationState,
    con: &'a mut SqliteConnection,
}

impl<'a> CleanupDataSource<'a> {
    pub fn new(state: &'a mut SimulationState, con: &'a mut SqliteConnection) -> Self {
        Self { state, con }
    }

    /// List of stored pipeline IDs ordered by insertion time.
    /// Last item in slice equals latest insertion.
    pub fn pipelines_ids(&self) -> Keys<PipelineID, Pipeline> {
        self.state.pipelines.keys()
    }

    /// List all pipelines in the order of insertion.
    pub fn pipelines(&self) -> Values<PipelineID, Pipeline> {
        self.state.pipelines.values()
    }

    pub async fn merge_request_events(
        &mut self,
        pipeline: PipelineID,
    ) -> Result<Vec<MergeRequestEvent>> {
        // SELECT * FROM pipeline LEFT JOIN MergeRequestEvent ON MergeRequestEvent.sourceBranch=Pipeline.ref WHERE Pipeline.id=257457 ORDER BY timestamp;
        // TODO Filter by timestamp < simulationTime

        let events = sqlx::query_as::<_, MergeRequestEvent>("SELECT * FROM pipeline LEFT JOIN MergeRequestEvent ON MergeRequestEvent.sourceBranch=Pipeline.ref WHERE Pipeline.id=$1 AND CAST(strftime('%s', timestamp) AS integer) <= CAST(strftime('%s', $2) AS integer) ORDER BY timestamp")
            .bind(pipeline)
            .bind(self.state.current_time)
            .fetch_all(&mut *self.con).await?;

        Ok(events)
    }
}
