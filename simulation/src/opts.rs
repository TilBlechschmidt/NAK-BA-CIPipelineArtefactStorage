use bytesize::ByteSize;
use clap::Clap;
use std::{collections::HashMap, path::PathBuf};

use crate::{
    algorithms::*,
    implementation::{CleanupAlgorithm, CleanupAttemptAlgorithm, FallbackCleanupAlgorithm},
    SimulationSpecification,
};

macro_rules! algorithm_map {
    ($t: ty, $( $name: expr => $algorithm: expr ),*) => {{
        let mut map: HashMap<String, Box<$t>> = ::std::collections::HashMap::new();
        $(map.insert($name.to_owned(), Box::new($algorithm)); )*
        map
    }}
}

#[derive(Clap, Clone)]
#[clap(version = "1.0", author = "Til B. <til@blechschmidt.de>")]
pub struct Opts {
    /// Seed to use for the simulation and any RNG based algorithms. Note that each algorithm will have its own PRNG instance based on the seed.
    #[clap(short, long, default_value = "1337")]
    pub seed: u64,
    /// Database that serves as the input to the simulation
    #[clap(short, long, default_value = "../data/out/simulation.db")]
    pub database_path: String,
    /// Directory in which to store the CSV statistics
    #[clap(
        short,
        long,
        parse(from_os_str),
        default_value = "../data/out/simulation/"
    )]
    pub output_directory: PathBuf,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Clap, Clone)]
pub enum SubCommand {
    OneShot(OneShotOpts),
    Batch(BatchOpts),
    SizeRamp(SizeRampOpts),
    GenerateML(GenerateML),
    GenerateStaticML(GenerateStaticML),
}

#[derive(Clap, Clone)]
pub struct GenerateML {}

#[derive(Clap, Clone)]
pub struct GenerateStaticML {}

#[derive(Clap, Clone)]
pub struct BatchOpts {
    /// Size limit for the simulated disk in GB
    size_limit: u64,
    /// Batch run definitions (definitions are concatenated algorithms e.g. 'MERGED-LRU-FIFO')
    definitions: Vec<String>,
}

impl BatchOpts {
    pub fn size_limit(&self) -> ByteSize {
        ByteSize::gb(self.size_limit)
    }

    pub fn specifications(self, output_folder: PathBuf) -> Vec<SimulationSpecification> {
        let storage_limit = self.size_limit();
        self.definitions
            .into_iter()
            .map(|definition| {
                let algorithms = definition.split("-").map(|s| s.to_owned()).collect();

                SimulationSpecification {
                    storage_limit,
                    algorithms,
                    output_path: output_folder.join(format!("{}.csv", definition)),
                    name: definition,
                }
            })
            .collect()
    }
}

#[derive(Clap, Clone)]
pub struct SizeRampOpts {
    /// Lower exponent for the simulated disk (provided value will be used like this: 2^lower_power)
    lower_exponent: u32,
    /// Upper exponent for the simulated disk (used like lower_exponent)
    upper_exponent: u32,
    /// Batch run definitions (definitions are concatenated algorithms e.g. 'MERGED-LRU-FIFO')
    definitions: Vec<String>,
}

impl SizeRampOpts {
    pub fn specifications(self, output_folder: PathBuf) -> Vec<SimulationSpecification> {
        let mut specifications = Vec::new();

        for exp in self.lower_exponent..self.upper_exponent {
            let storage_limit = ByteSize::gb(2u64.pow(exp));
            let limit_name = storage_limit.to_string().replace(" ", "").replace(".0", "");
            let size_directory = output_folder.join(&limit_name);

            for definition in self.definitions.iter() {
                let algorithms = definition.split("-").map(|s| s.to_owned()).collect();
                let name = format!("{}-{}", definition, limit_name);

                let output_path = size_directory.join(format!("{}.csv", definition));

                specifications.push(SimulationSpecification {
                    storage_limit,
                    algorithms,
                    output_path,
                    name,
                });
            }
        }

        specifications
    }
}

#[derive(Clap, Clone)]
pub struct OneShotOpts {
    /// Size limit for the simulated disk in GB
    size_limit: u64,
    /// List of algorithms.
    /// If you provide more than one algorithm then all but the last have to be "AttemptAlgorithms".
    algorithms: Vec<String>,
    /// Name of the output file
    #[clap(short, long, default_value = "one_shot.csv")]
    pub filename: String,
}

impl OneShotOpts {
    pub fn size_limit(&self) -> ByteSize {
        ByteSize::gb(self.size_limit)
    }

    pub fn specification(self, output_folder: PathBuf) -> SimulationSpecification {
        SimulationSpecification {
            name: self.algorithms.join("-"),
            storage_limit: self.size_limit(),
            algorithms: self.algorithms,
            output_path: output_folder.join(self.filename),
        }
    }
}

pub fn build_algorithm(algorithms: &Vec<String>, seed: u64) -> Box<FallbackCleanupAlgorithm> {
    let fallback_algorithm_string = algorithms
        .last()
        .expect("You must provide at least one algorithm");

    let mut failable_algorithms = algorithm_map![dyn CleanupAttemptAlgorithm,
        "MERGED" => BranchMergedAlgorithm {},
        "LRU" => LRUAlgorithm {},
        "MRU" => MRUAlgorithm {},
        "MRU.2" => MRURangedAlgorithm::new(2),
        "MRU.4" => MRURangedAlgorithm::new(4),
        "MRU.8" => MRURangedAlgorithm::new(8),
        "MRU.16" => MRURangedAlgorithm::new(16),
        "MRU.32" => MRURangedAlgorithm::new(32),
        "MRU.64" => MRURangedAlgorithm::new(64),
        "LF" => LargestFirstAlgorithm {},
        "SF" => SmallestFirstAlgorithm {},
        "STATUS" => LayeredStatusAlgorithm {}
    ];

    let mut fallback_algorithms = algorithm_map![dyn CleanupAlgorithm,
        "RAND" => RandomAlgorithm::new(seed),
        "LIFO" => LIFOAlgorithm {},
        "FIFO" => FIFOAlgorithm {},
        "SCORE.DEFAULT" => ScoringAlgorithmManager::new(vec![
            Box::new(StatusAlgorithm::default()),
            Box::new(MergedAlgorithm::default()),
            Box::new(AgeAlgorithm::default())
        ]),
        "SCORE" => ScoringAlgorithmManager::new(vec![
            Box::new(StatusAlgorithm::new(0, 45, -5, 0)),
            Box::new(MergedAlgorithm::new(30)),
            Box::new(AgeAlgorithm::new(60 * 60 * 24 * 3, 50))
        ])
    ];

    let algorithms = algorithms
        .iter()
        .take(algorithms.len() - 1)
        .map(|key| {
            failable_algorithms
                .remove(key)
                .expect(&format!("Algorithm '{}' not found!", key))
        })
        .collect::<Vec<Box<dyn CleanupAttemptAlgorithm>>>();

    let fallback_algorithm = fallback_algorithms
        .remove(fallback_algorithm_string)
        .expect(&format!(
            "Fallback algorithm '{}' not found!",
            fallback_algorithm_string
        ));

    Box::new(FallbackCleanupAlgorithm::new(
        algorithms,
        fallback_algorithm,
    ))
}
