use super::{
    data_source::{SimulationEvent, SimulationEventKind},
    state::SimulationState,
};
use anyhow::Result;
use bytesize::ByteSize;
use std::{fmt, fs::create_dir_all, fs::File, io::Write, path::PathBuf};

pub struct DataPoint {
    event: SimulationEvent,

    occupied_storage: ByteSize,
    stored_pipeline_count: usize,

    access_count: u32,
    access_count_missed: u32,

    deleted_count: u32,
}

impl DataPoint {
    pub fn new(state: &SimulationState) -> Self {
        Self {
            event: state.latest_event.unwrap(),
            occupied_storage: state.occupied_storage,
            stored_pipeline_count: state.stored_pipelines.len(),
            access_count: state.access_count,
            access_count_missed: state.access_count_missed,
            deleted_count: state.deleted_count,
        }
    }

    pub fn missed_percentage(&self) -> f64 {
        let missed: f64 = self.access_count_missed.into();
        let total: f64 = self.access_count.into();

        if total == 0.0 {
            return 0.0;
        }

        missed / total
    }

    pub fn csv_header() -> &'static str {
        "Occupied storage,Stored count,Deleted count,Access count,Missed access count,Missed fraction"
    }
}

impl fmt::Display for DataPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{},{},{},{}",
            self.occupied_storage.as_u64(),
            self.stored_pipeline_count,
            self.deleted_count,
            self.access_count,
            self.access_count_missed,
            self.missed_percentage()
        )
    }
}

pub struct Statistics {
    data_points: Vec<DataPoint>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            data_points: Vec::new(),
        }
    }

    pub fn record(&mut self, state: &SimulationState) {
        self.data_points.push(DataPoint::new(state));
    }

    pub fn current_miss_percentage(&self) -> f64 {
        if let Some(data_point) = self.data_points.last() {
            data_point.missed_percentage()
        } else {
            0.0
        }
    }

    pub fn write_csv(&self, path: PathBuf) -> Result<()> {
        create_dir_all(path.parent().unwrap())?;
        let mut f = File::create(path)?;
        f.write_fmt(format_args!("{}", self))?;

        Ok(())
    }
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", DataPoint::csv_header())?;

        for data_point in self
            .data_points
            .iter()
            .filter(|e| e.event.kind == SimulationEventKind::PipelineFinished)
        {
            writeln!(f, "{}", data_point)?;
        }

        Ok(())
    }
}
