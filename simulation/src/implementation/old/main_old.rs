use anyhow::Result;
use bytesize::ByteSize;
use sqlx::{Connection, SqliteConnection};

mod algorithms;
mod implementation;

use algorithms::{
    BranchMergedAlgorithm, FIFOAlgorithm, LIFOAlgorithm, LRUAlgorithm, LargestFirstAlgorithm,
    SmallestFirstAlgorithm,
};
use implementation::{Simulation, SimulationState};


// IDEA: Event based cleanup algorithm (more specifically: onMergeEvent)
//       Would at least be interesting in terms of disk usage target

#[tokio::main]
async fn main() -> Result<()> {
    let algo = BranchMergedAlgorithm::new(LRUAlgorithm::new(FIFOAlgorithm {}));
    let simulation = Simulation::new(algo);

    let mut state = SimulationState::new(1337, ByteSize::gb(512)).await?;
    let mut con = SqliteConnection::connect("../data/out/simulation.db").await?;

    loop {
        match simulation.advance(&mut state, &mut con).await {
            Err(e) => {
                eprintln!("{:?}", e);
                break;
            }
            Ok(_) => {
                println!(
                    "{}\t{}\t{}\t{}%",
                    state.pipelines.len(),
                    ByteSize::b(state.bytes),
                    state.deleted_pipeline_count(),
                    state.missed_percentage() * 100.0
                );
            }
        }
    }

    println!("Reached final size of {}", ByteSize::b(state.bytes));
    println!("Pipelines: {}", state.pipelines.len());
    println!(
        "Missed {}% ({}/{}) of all accesses",
        state.missed_percentage() * 100.0,
        state.access_count_missed,
        state.access_count
    );

    Ok(())
}
