//! Standalone Basic Throughput Test
//! This test mimics the basic_throughput.yaml configuration without external dependencies

use std::time::{Duration, Instant};
use std::thread;

#[derive(Debug)]
struct BenchmarkResults {
    test_name: String,
    duration: Duration,
    total_transactions: u64,
    successful_transactions: u64,
    failed_transactions: u64,
    actual_tps: f64,
    success_rate: f64,
    target_tps: f64,
}

struct BasicThroughputTest {
    test_name: String,
    validators: u32,
    target_tps: f64,
    duration: Duration,
    concurrent_connections: u32,
    transaction_types: Vec<String>,
}

impl BasicThroughputTest {
    fn new() -> Self {
        Self {
            test_name: "Basic Throughput Test".to_string(),
            validators: 4,
            target_tps: 1000.0,
            duration: Duration::from_secs(10), // 10 seconds for demo
            concurrent_connections: 100,
            transaction_types: vec!["transfer".to_string(), "erc20".to_string()],
        }
    }

    fn run(&self) -> BenchmarkResults {
        println!("🚀 Starting IPC Basic Throughput Test");
        println!("=====================================");
        println!("Test: {}", self.test_name);
        println!("Validators: {}", self.validators);
        println!("Target TPS: {}", self.target_tps);
        println!("Duration: {:?}", self.duration);
        println!("Concurrent Connections: {}", self.concurrent_connections);
        println!("Transaction Types: {:?}", self.transaction_types);
        println!("=====================================");

        let start_time = Instant::now();
        let mut total_transactions = 0u64;
        let mut successful_transactions = 0u64;
        let mut failed_transactions = 0u64;

        // Calculate transaction interval based on target TPS and concurrent connections
        let tx_interval = Duration::from_millis(
            (1000.0 / self.target_tps * self.concurrent_connections as f64) as u64
        );

        println!("Transaction interval: {:?}", tx_interval);

        let mut handles = Vec::new();

        for i in 0..self.concurrent_connections {
            let duration = self.duration;
            let interval = tx_interval;
            let primary_tx_type = self.transaction_types[0].clone();

            let handle = thread::spawn(move || {
                let mut local_total = 0u64;
                let mut local_successful = 0u64;
                let mut local_failed = 0u64;

                let start = Instant::now();
                let mut next_tx_time = start;

                if i == 0 {
                    println!("Worker {} started (first worker)", i);
                }

                while start.elapsed() < duration {
                    if Instant::now() >= next_tx_time {
                        // Simulate transaction processing
                        let tx_result = simulate_transaction(i, &primary_tx_type);
                        local_total += 1;

                        if tx_result {
                            local_successful += 1;
                        } else {
                            local_failed += 1;
                        }

                        next_tx_time += interval;

                        // Print progress every 1000 transactions for first worker
                        if i == 0 && local_total % 1000 == 0 {
                            println!("Worker {} - Transactions: {}", i, local_total);
                        }
                    }

                    // Small sleep to prevent busy waiting
                    thread::sleep(Duration::from_millis(1));
                }

                if i == 0 {
                    println!("Worker {} finished - Total: {}, Success: {}, Failed: {}",
                             i, local_total, local_successful, local_failed);
                }

                (local_total, local_successful, local_failed)
            });

            handles.push(handle);
        }

        println!("All workers started. Running test...");

        // Wait for all threads to complete
        for handle in handles {
            if let Ok((t, s, f)) = handle.join() {
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

        BenchmarkResults {
            test_name: self.test_name.clone(),
            duration: actual_duration,
            total_transactions,
            successful_transactions,
            failed_transactions,
            actual_tps,
            success_rate,
            target_tps: self.target_tps,
        }
    }
}

fn simulate_transaction(worker_id: u32, tx_type: &str) -> bool {
    // Simulate different transaction types with different processing times
    let work_duration = match tx_type {
        "transfer" => Duration::from_micros(50 + (worker_id % 10) as u64),
        "erc20" => Duration::from_micros(80 + (worker_id % 15) as u64),
        "contract_call" => Duration::from_micros(120 + (worker_id % 20) as u64),
        _ => Duration::from_micros(100 + (worker_id % 10) as u64),
    };

    thread::sleep(work_duration);

    // Simulate 95% success rate
    worker_id % 20 != 0
}

fn print_results(results: &BenchmarkResults) {
    println!("\n🎯 Basic Throughput Test Results");
    println!("=================================");
    println!("Test: {}", results.test_name);
    println!("Duration: {:?}", results.duration);
    println!("Total Transactions: {}", results.total_transactions);
    println!("Successful Transactions: {}", results.successful_transactions);
    println!("Failed Transactions: {}", results.failed_transactions);
    println!("Actual TPS: {:.2}", results.actual_tps);
    println!("Success Rate: {:.2}%", results.success_rate * 100.0);
    println!("Target TPS: {:.2}", results.target_tps);
    println!("TPS Efficiency: {:.2}%", (results.actual_tps / results.target_tps) * 100.0);

    println!("\n🔍 Analysis:");
    if results.actual_tps >= results.target_tps * 0.9 {
        println!("✅ PASSED: Achieved target TPS within 10% tolerance");
    } else if results.actual_tps >= results.target_tps * 0.7 {
        println!("⚠️  PARTIAL: Achieved 70-90% of target TPS");
    } else {
        println!("❌ FAILED: Below 70% of target TPS");
    }

    if results.success_rate >= 0.95 {
        println!("✅ PASSED: High success rate ({}%)", (results.success_rate * 100.0) as u32);
    } else if results.success_rate >= 0.85 {
        println!("⚠️  PARTIAL: Moderate success rate ({}%)", (results.success_rate * 100.0) as u32);
    } else {
        println!("❌ FAILED: Low success rate ({}%)", (results.success_rate * 100.0) as u32);
    }

    // Overall assessment
    let tps_score = (results.actual_tps / results.target_tps).min(1.0);
    let success_score = results.success_rate;
    let overall_score = (tps_score + success_score) / 2.0;

    println!("\n🏆 Overall Assessment:");
    println!("TPS Score: {:.2}/1.0", tps_score);
    println!("Success Score: {:.2}/1.0", success_score);
    println!("Overall Score: {:.2}/1.0", overall_score);

    if overall_score >= 0.9 {
        println!("✅ EXCELLENT: Benchmark performance is excellent!");
    } else if overall_score >= 0.8 {
        println!("✅ GOOD: Benchmark performance is good");
    } else if overall_score >= 0.6 {
        println!("⚠️  FAIR: Benchmark performance is fair");
    } else {
        println!("❌ POOR: Benchmark performance needs improvement");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IPC Basic Throughput Test");
    println!("Simulating basic_throughput.yaml configuration");
    println!("Testing 4 validators, 1000 TPS target, 10-second duration");
    println!("This test uses simulation - not actual blockchain transactions");
    println!();

    let test = BasicThroughputTest::new();
    let results = test.run();

    print_results(&results);

    Ok(())
}