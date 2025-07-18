//! Simple throughput benchmark for testing basic functionality

use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct SimpleBenchmarkConfig {
    pub duration: Duration,
    pub target_tps: f64,
    pub concurrent_connections: usize,
}

impl Default for SimpleBenchmarkConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(30),
            target_tps: 100.0,
            concurrent_connections: 10,
        }
    }
}

#[derive(Debug)]
pub struct SimpleBenchmarkResults {
    pub duration: Duration,
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub actual_tps: f64,
    pub success_rate: f64,
}

pub struct SimpleBenchmark {
    config: SimpleBenchmarkConfig,
}

impl SimpleBenchmark {
    pub fn new(config: SimpleBenchmarkConfig) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> SimpleBenchmarkResults {
        println!("Starting simple throughput benchmark...");
        println!("Duration: {:?}", self.config.duration);
        println!("Target TPS: {}", self.config.target_tps);
        println!("Concurrent connections: {}", self.config.concurrent_connections);

        let start_time = Instant::now();
        let mut total_transactions = 0u64;
        let mut successful_transactions = 0u64;
        let mut failed_transactions = 0u64;

        let tx_interval = Duration::from_millis(
            (1000.0 / self.config.target_tps * self.config.concurrent_connections as f64) as u64
        );

        let mut tasks = Vec::new();
        for i in 0..self.config.concurrent_connections {
            let duration = self.config.duration;
            let interval = tx_interval;

            let task = tokio::spawn(async move {
                let mut local_total = 0u64;
                let mut local_successful = 0u64;
                let mut local_failed = 0u64;

                let start = Instant::now();
                let mut next_tx_time = start;

                while start.elapsed() < duration {
                    if Instant::now() >= next_tx_time {
                        // Simulate transaction processing
                        let tx_result = simulate_transaction(i).await;
                        local_total += 1;

                        if tx_result {
                            local_successful += 1;
                        } else {
                            local_failed += 1;
                        }

                        next_tx_time += interval;
                    }

                    // Small sleep to prevent busy waiting
                    sleep(Duration::from_millis(1)).await;
                }

                (local_total, local_successful, local_failed)
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        for task in tasks {
            if let Ok((t, s, f)) = task.await {
                total_transactions += t;
                successful_transactions += s;
                failed_transactions += f;
            }
        }

        let actual_duration = start_time.elapsed();
        let actual_tps = total_transactions as f64 / actual_duration.as_secs_f64();
        let success_rate = if total_transactions > 0 {
            successful_transactions as f64 / total_transactions as f64
        } else {
            0.0
        };

        SimpleBenchmarkResults {
            duration: actual_duration,
            total_transactions,
            successful_transactions,
            failed_transactions,
            actual_tps,
            success_rate,
        }
    }
}

async fn simulate_transaction(connection_id: usize) -> bool {
    // Simulate some work with random success/failure
    let work_duration = Duration::from_millis(10 + (connection_id % 5) as u64);
    sleep(work_duration).await;

    // Simulate 95% success rate
    connection_id % 20 != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_benchmark() {
        let config = SimpleBenchmarkConfig {
            duration: Duration::from_secs(5),
            target_tps: 50.0,
            concurrent_connections: 5,
        };

        let benchmark = SimpleBenchmark::new(config);
        let results = benchmark.run().await;

        assert!(results.total_transactions > 0);
        assert!(results.actual_tps > 0.0);
        assert!(results.success_rate > 0.8);
        println!("Test results: {:?}", results);
    }
}