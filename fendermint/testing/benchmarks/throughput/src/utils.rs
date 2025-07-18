// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Utility functions for benchmarking

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let millis = duration.subsec_millis();

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else if seconds > 0 {
        format!("{}.{:03}s", seconds, millis)
    } else {
        format!("{}ms", millis)
    }
}

/// Format bytes in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Format number with thousands separators
pub fn format_number(num: u64) -> String {
    let s = num.to_string();
    let bytes = s.as_bytes();
    let mut result = String::new();

    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(b as char);
    }

    result
}

/// Get current timestamp
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Parse duration from string (e.g., "30s", "5m", "1h")
pub fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(Duration::from_secs(0));
    }

    let (number_str, unit) = if s.ends_with("ms") {
        (&s[..s.len() - 2], "ms")
    } else if s.ends_with('s') {
        (&s[..s.len() - 1], "s")
    } else if s.ends_with('m') {
        (&s[..s.len() - 1], "m")
    } else if s.ends_with('h') {
        (&s[..s.len() - 1], "h")
    } else {
        (s, "s") // Default to seconds
    };

    let number: u64 = number_str.parse()
        .map_err(|_| anyhow::anyhow!("Invalid duration format: {}", s))?;

    let duration = match unit {
        "ms" => Duration::from_millis(number),
        "s" => Duration::from_secs(number),
        "m" => Duration::from_secs(number * 60),
        "h" => Duration::from_secs(number * 3600),
        _ => return Err(anyhow::anyhow!("Unknown time unit: {}", unit)),
    };

    Ok(duration)
}

/// Calculate percentile from sorted values
pub fn calculate_percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let index = (percentile / 100.0) * (values.len() - 1) as f64;
    let lower_index = index.floor() as usize;
    let upper_index = index.ceil() as usize;

    if lower_index == upper_index {
        values[lower_index]
    } else {
        let weight = index - lower_index as f64;
        values[lower_index] * (1.0 - weight) + values[upper_index] * weight
    }
}

/// Generate a simple progress bar
pub fn progress_bar(current: u64, total: u64, width: usize) -> String {
    if total == 0 {
        return "█".repeat(width);
    }

    let progress = current as f64 / total as f64;
    let filled = (progress * width as f64) as usize;
    let empty = width - filled;

    format!("{}{}",
        "█".repeat(filled),
        "░".repeat(empty)
    )
}

/// Create a summary table for results
pub fn create_summary_table(results: &[(&str, &str)]) -> String {
    let max_key_len = results.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let max_val_len = results.iter().map(|(_, v)| v.len()).max().unwrap_or(0);

    let mut table = String::new();
    let border = format!("┌{}┬{}┐\n", "─".repeat(max_key_len + 2), "─".repeat(max_val_len + 2));
    table.push_str(&border);

    for (i, (key, value)) in results.iter().enumerate() {
        let row = format!("│ {:width_key$} │ {:width_val$} │\n",
                         key, value,
                         width_key = max_key_len,
                         width_val = max_val_len);
        table.push_str(&row);

        if i < results.len() - 1 {
            let separator = format!("├{}┼{}┤\n", "─".repeat(max_key_len + 2), "─".repeat(max_val_len + 2));
            table.push_str(&separator);
        }
    }

    let bottom_border = format!("└{}┴{}┘\n", "─".repeat(max_key_len + 2), "─".repeat(max_val_len + 2));
    table.push_str(&bottom_border);

    table
}

/// Benchmark configuration validator
pub fn validate_benchmark_config(
    validators: usize,
    duration: Duration,
    concurrent_users: usize,
    target_tps: u64,
) -> Result<()> {
    if validators == 0 {
        return Err(anyhow::anyhow!("Number of validators must be greater than 0"));
    }

    if validators > 100 {
        return Err(anyhow::anyhow!("Number of validators should not exceed 100"));
    }

    if duration.as_secs() == 0 {
        return Err(anyhow::anyhow!("Duration must be greater than 0"));
    }

    if concurrent_users == 0 {
        return Err(anyhow::anyhow!("Number of concurrent users must be greater than 0"));
    }

    if target_tps == 0 {
        return Err(anyhow::anyhow!("Target TPS must be greater than 0"));
    }

    // Warn about potentially unrealistic configurations
    if target_tps > 10000 {
        eprintln!("Warning: Target TPS {} is very high and may not be achievable", target_tps);
    }

    if concurrent_users > 1000 {
        eprintln!("Warning: {} concurrent users is very high and may cause resource issues", concurrent_users);
    }

    Ok(())
}

/// Calculate TPS from transaction count and duration
pub fn calculate_tps(transaction_count: u64, duration: Duration) -> f64 {
    if duration.as_secs_f64() == 0.0 {
        return 0.0;
    }
    transaction_count as f64 / duration.as_secs_f64()
}

/// Calculate success rate percentage
pub fn calculate_success_rate(successful: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (successful as f64 / total as f64) * 100.0
}

/// Generate random transaction data
pub fn generate_random_data(size: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}

/// Simple moving average calculator
pub struct MovingAverage {
    window_size: usize,
    values: Vec<f64>,
    sum: f64,
    index: usize,
}

impl MovingAverage {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            values: vec![0.0; window_size],
            sum: 0.0,
            index: 0,
        }
    }

    pub fn add(&mut self, value: f64) {
        self.sum -= self.values[self.index];
        self.values[self.index] = value;
        self.sum += value;
        self.index = (self.index + 1) % self.window_size;
    }

    pub fn average(&self) -> f64 {
        self.sum / self.window_size as f64
    }
}

/// Rate limiter for controlling TPS
pub struct RateLimiter {
    target_tps: u64,
    last_time: SystemTime,
    tokens: f64,
    max_tokens: f64,
}

impl RateLimiter {
    pub fn new(target_tps: u64) -> Self {
        Self {
            target_tps,
            last_time: SystemTime::now(),
            tokens: target_tps as f64,
            max_tokens: target_tps as f64,
        }
    }

    pub fn acquire(&mut self) -> Result<()> {
        let now = SystemTime::now();
        let elapsed = now.duration_since(self.last_time).unwrap_or_default();

        // Add tokens based on elapsed time
        let tokens_to_add = elapsed.as_secs_f64() * self.target_tps as f64;
        self.tokens = (self.tokens + tokens_to_add).min(self.max_tokens);
        self.last_time = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Rate limit exceeded"))
        }
    }

    pub fn set_target_tps(&mut self, target_tps: u64) {
        self.target_tps = target_tps;
        self.max_tokens = target_tps as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_secs(30)), "30.000s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
    }

    #[test]
    fn test_calculate_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_percentile(&values, 50.0), 3.0);
        assert_eq!(calculate_percentile(&values, 90.0), 4.6);
        assert_eq!(calculate_percentile(&values, 100.0), 5.0);
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(5, 10, 10), "█████░░░░░");
        assert_eq!(progress_bar(10, 10, 10), "██████████");
        assert_eq!(progress_bar(0, 10, 10), "░░░░░░░░░░");
    }

    #[test]
    fn test_calculate_tps() {
        assert_eq!(calculate_tps(1000, Duration::from_secs(10)), 100.0);
        assert_eq!(calculate_tps(0, Duration::from_secs(10)), 0.0);
        assert_eq!(calculate_tps(1000, Duration::from_secs(0)), 0.0);
    }

    #[test]
    fn test_calculate_success_rate() {
        assert_eq!(calculate_success_rate(90, 100), 90.0);
        assert_eq!(calculate_success_rate(0, 100), 0.0);
        assert_eq!(calculate_success_rate(100, 100), 100.0);
        assert_eq!(calculate_success_rate(0, 0), 0.0);
    }

    #[test]
    fn test_moving_average() {
        let mut ma = MovingAverage::new(3);
        ma.add(1.0);
        ma.add(2.0);
        ma.add(3.0);
        assert_eq!(ma.average(), 2.0);

        ma.add(4.0);
        assert_eq!(ma.average(), 3.0); // (2+3+4)/3
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(10); // 10 TPS

        // Should be able to acquire initially
        assert!(limiter.acquire().is_ok());

        // Should be rate limited if called immediately
        assert!(limiter.acquire().is_err());
    }

    #[test]
    fn test_validate_benchmark_config() {
        // Valid config
        assert!(validate_benchmark_config(4, Duration::from_secs(300), 100, 1000).is_ok());

        // Invalid configs
        assert!(validate_benchmark_config(0, Duration::from_secs(300), 100, 1000).is_err());
        assert!(validate_benchmark_config(4, Duration::from_secs(0), 100, 1000).is_err());
        assert!(validate_benchmark_config(4, Duration::from_secs(300), 0, 1000).is_err());
        assert!(validate_benchmark_config(4, Duration::from_secs(300), 100, 0).is_err());
    }
}