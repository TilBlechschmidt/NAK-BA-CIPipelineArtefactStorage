use super::{
    data_source::{DataSource, MergeRequestEvent, SimulationEvent, SimulationEventKind},
    CleanupAlgorithm, CleanupDataSource, PipelineID,
};
use anyhow::{anyhow, bail, Result};
use bytesize::ByteSize;
use std::collections::{BTreeSet, HashMap};

pub struct SimulationState {
    pub latest_event: Option<SimulationEvent>,

    data_source: DataSource,
    algorithm: Box<dyn CleanupAlgorithm>,

    pub storage_limit: ByteSize,
    pub occupied_storage: ByteSize,
    pub stored_pipelines: BTreeSet<PipelineID>,

    pub access_count: u32,
    pub access_count_missed: u32,
    pub deleted_count: u32,

    /// Timestamps of accesses to each pipeline
    pub accesses: HashMap<PipelineID, Vec<i64>>,

    /// IDs of merged pipelines in chronological order
    pub merges: BTreeSet<PipelineID>,

    /// Timestamps of when a pipeline was created
    pub storage_times: HashMap<PipelineID, i64>,
}

impl SimulationState {
    pub fn new(
        data_source: &DataSource,
        algorithm: Box<dyn CleanupAlgorithm>,
        storage_limit: ByteSize,
    ) -> Self {
        Self {
            latest_event: None,
            data_source: data_source.clone(),
            algorithm,
            storage_limit,
            occupied_storage: ByteSize::b(0),
            stored_pipelines: BTreeSet::new(),
            access_count: 0,
            access_count_missed: 0,
            deleted_count: 0,
            accesses: HashMap::new(),
            merges: BTreeSet::new(),
            storage_times: HashMap::new(),
        }
    }

    async fn remove_pipeline(&mut self, id: &PipelineID) -> Result<bool> {
        let was_present = self.stored_pipelines.remove(id);
        self.merges.remove(id);

        if was_present {
            self.deleted_count += 1;
            // TODO This is really ugly. Fix it by implementing the sub and sub-assign traits.
            self.occupied_storage = ByteSize::b(
                self.occupied_storage.as_u64()
                    - self.data_source.size_of_pipeline(*id).await?.as_u64(),
            );
        }

        Ok(was_present)
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        let mut i = 0;

        while self.is_over_limit() {
            let data_source = CleanupDataSource::new(self, &self.data_source);
            let pipeline_to_purge = self.algorithm.select_pipeline(&data_source).await;
            self.remove_pipeline(&pipeline_to_purge).await?;

            if i > 10_000 {
                bail!(
                    "Algorithm did not manage to get below limit after {} iterations",
                    i
                );
            }
            i += 1;
        }

        Ok(())
    }

    pub async fn process(&mut self, event: SimulationEvent) -> Result<()> {
        match event.kind {
            SimulationEventKind::MergeRequestEvent => {
                let mr_event: MergeRequestEvent =
                    self.data_source.merge_request_event(event.key).await?;

                if mr_event.action == "merge" {
                    if let Ok(pipelines) = self
                        .data_source
                        .pipelines_for_ref(mr_event.source_branch)
                        .await
                    {
                        for pipeline in pipelines {
                            if self.stored_pipelines.contains(&pipeline.id) {
                                self.merges.insert(pipeline.id);
                            }
                        }
                    }
                }
            }
            SimulationEventKind::Access => {
                match self.data_source.access_log_entry(event.key).await {
                    Ok(entry) => {
                        if self
                            .data_source
                            .pipeline_is_populated(entry.pipeline)
                            .await?
                        {
                            self.access_count += 1;

                            if !self.stored_pipelines.contains(&entry.pipeline) {
                                self.access_count_missed += 1;
                                // eprintln!(
                                //     "Missed access {} to pipeline {}",
                                //     event.key, entry.pipeline
                                // );
                            }

                            if let Some(accesses) = self.accesses.get_mut(&entry.pipeline) {
                                accesses.push(entry.timestamp);
                            } else {
                                self.accesses.insert(entry.pipeline, vec![entry.timestamp]);
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to locate access log entry: {:?}", e),
                }
            }
            SimulationEventKind::PipelineCreated => {
                if let Err(_) = self.data_source.size_of_pipeline(event.key).await {
                    // println!("Skipping pipeline due to unavailable size samples.");
                } else {
                    // println!("Taking pipeline");
                    if !self.stored_pipelines.insert(event.key) {
                        eprintln!(
                            "Attempting to store pipeline which is already stored ({})",
                            event.key
                        );
                    } else {
                        self.storage_times.insert(
                            event.key,
                            self.latest_event.map(|e| e.timestamp).unwrap_or(0),
                        );
                    }
                }
            }
            SimulationEventKind::PipelineFinished => {
                if let Ok(size) = self.data_source.size_of_pipeline(event.key).await {
                    self.occupied_storage = self.occupied_storage + size;
                }
            }
        }

        self.latest_event = Some(event);

        Ok(())
    }

    pub fn is_over_limit(&self) -> bool {
        self.occupied_storage > self.storage_limit
    }

    pub async fn remove_pipelines_older_than(&mut self, age: i64) -> Result<()> {
        let timestamp = self
            .latest_event
            .ok_or(anyhow!("Simulation has no latest event"))?
            .timestamp;

        let ids = self
            .stored_pipelines
            .iter()
            .filter(|id| {
                if let Some(storage_time) = self.storage_times.get(*id) {
                    timestamp - storage_time > age
                } else {
                    false
                }
            })
            .map(|i| *i)
            .collect::<Vec<_>>();

        for id in ids {
            self.remove_pipeline(&id).await?;
        }

        Ok(())
    }
}
