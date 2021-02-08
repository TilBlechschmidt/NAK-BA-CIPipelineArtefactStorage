use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use rand::{
    distributions::Uniform,
    prelude::{Distribution, StdRng},
    SeedableRng,
};
use sqlx::SqlitePool;

fn split_first<'a>(from: &'a str, separator: &str) -> Option<(&'a str, &'a str)> {
    let mut components = from.split(separator).collect::<Vec<&str>>();

    if components.len() != 2 {
        return None;
    }

    let part2 = components.pop().unwrap();
    let part1 = components.pop().unwrap();

    return Some((part1, part2));
}

fn split_job<'a>(job: &'a str) -> Result<(String, &'a str)> {
    return split_first(job, ":")
        .map(|j| (j.0.replace("_reorg", ""), j.1))
        .ok_or_else(|| anyhow!("Unable to split job name: '{}'", job));
}

pub struct JobSizeSampler {
    rng: StdRng,
    distributions: HashMap<String, Uniform<i64>>,
}

impl JobSizeSampler {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            distributions: HashMap::new(),
        }
    }

    async fn sample_count(
        &self,
        environment: &str,
        test_suite: &str,
        con: &SqlitePool,
    ) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM JobSizeSample WHERE environment=$1 AND testSuite=$2",
        )
        .bind(environment)
        .bind(test_suite)
        .fetch_one(con)
        .await?;

        return Ok(row.0);
    }

    async fn ensure_distribution(&mut self, job: &str, con: &SqlitePool) -> Result<()> {
        if !self.distributions.contains_key(job) {
            let (environment, test_suite) = split_job(job)?;
            let count = self.sample_count(&environment, test_suite, con).await?;

            if count <= 30 {
                bail!("Not enough size samples available!");
            }

            let distribution = Uniform::new(0, count);
            self.distributions.insert(job.to_owned(), distribution);
        }

        Ok(())
    }

    pub async fn sample(&mut self, job: &str, con: &SqlitePool) -> Result<i64> {
        self.ensure_distribution(job, con).await?;
        let distribution = self
            .distributions
            .get(job)
            .ok_or_else(|| anyhow!("No distribution found for {}", job))?;

        let sample_index = distribution.sample(&mut self.rng);
        let (environment, test_suite) = split_job(job)?;
        let row: (i64,) = sqlx::query_as("SELECT bytes FROM JobSizeSample WHERE environment=$1 AND testSuite=$2 ORDER BY id LIMIT 1 OFFSET $3").bind(environment).bind(test_suite).bind(sample_index).fetch_one(con).await?;

        Ok(row.0)
    }
}
