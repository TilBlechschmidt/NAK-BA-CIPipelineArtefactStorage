#![feature(map_first_last)]
#![feature(result_flattening)]

use std::path::PathBuf;

use anyhow::Result;
use async_std::task;
use bytesize::ByteSize;
use clap::Clap;

mod algorithms;
mod implementation;
mod opts;

use implementation::{DataSource, MLGenerator, Simulation, StaticMLGenerator};
use indicatif::MultiProgress;
use opts::{build_algorithm, Opts, SubCommand};

// TODO Idea: Weighted/Cost based algorithm

// IDEA: Event based cleanup algorithm (more specifically: onMergeEvent)
//       Would at least be interesting in terms of disk usage target

pub struct SimulationSpecification {
    name: String,
    algorithms: Vec<String>,
    storage_limit: ByteSize,
    output_path: PathBuf,
}

async fn run_simulations(
    database_path: String,
    seed: u64,
    specifications: Vec<SimulationSpecification>,
) -> Result<()> {
    let progress_bar = MultiProgress::new();
    let mut handles = Vec::new();

    eprintln!("Pre-populating pipeline size samples ...");

    let data_source = DataSource::open(&database_path, seed).await?;
    let total_size = data_source.populate_size_samples().await?;
    let event_count = data_source.event_count().await?;

    eprintln!("Simulating a total pipeline volume of {} and {} events...", total_size, event_count);

    for specification in specifications {
        let simulation = Simulation::prepare(data_source.clone(), &progress_bar).await?;
        let algorithm = build_algorithm(&specification.algorithms, seed);

        simulation.set_name(&specification.name);

        handles.push(task::spawn(async move {
            simulation
                .run(algorithm, specification.storage_limit)
                .await
                .map(|statistics| statistics.write_csv(specification.output_path))
                .flatten()
        }))
    }

    task::spawn_blocking(move || {
        progress_bar.join().unwrap();
    })
    .await;

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();

    let opts: Opts = Opts::parse();
    let mut output_folder = opts.output_directory.clone();

    match opts.subcommand {
        SubCommand::OneShot(one_shot_opts) => {
            let specification = one_shot_opts.specification(output_folder);
            run_simulations(opts.database_path, opts.seed, vec![specification]).await?;
        }
        SubCommand::Batch(batch_opts) => {
            output_folder.push("batch");
            let specifications = batch_opts.specifications(output_folder);
            run_simulations(opts.database_path, opts.seed, specifications).await?;
        }
        SubCommand::SizeRamp(ramp_opts) => {
            output_folder.push("size-ramp");
            let specifications = ramp_opts.specifications(output_folder);
            run_simulations(opts.database_path, opts.seed, specifications).await?;
        }
        SubCommand::GenerateML(_generate_opts) => {
            output_folder.push("ml-data.csv");
            let generator = MLGenerator::prepare(&opts.database_path, opts.seed).await?;
            generator.generate().await?;
        }
        SubCommand::GenerateStaticML(_generate_opts) => {
            output_folder.push("ml-data.csv");
            let generator = StaticMLGenerator::new(&opts.database_path, opts.seed).await?;
            generator.generate().await?;
        }
    }

    Ok(())
}
