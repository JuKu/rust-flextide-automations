//! Backup Management Module
//!
//! Provides functionality for managing backups, backup jobs, and backup operations.

mod backup;
pub mod database;
mod error;
mod execution;

use chrono::{DateTime, Utc};
use std::str::FromStr;

pub use backup::*;
pub use database::*;
pub use error::BackupError;
pub use execution::*;

/// Normalize cron schedule to 6 fields (add seconds if 5 fields provided)
/// 
/// The cron crate 0.15 expects 6 fields: second minute hour day month weekday
/// This function converts 5-field expressions (minute hour day month weekday) to 6 fields
/// by prepending "0" for seconds.
fn normalize_cron_schedule(schedule: &str) -> String {
    let schedule = schedule.trim();
    let parts: Vec<&str> = schedule.split_whitespace().collect();
    
    match parts.len() {
        5 => {
            // 5 fields: minute hour day month weekday -> prepend "0" for seconds
            format!("0 {}", schedule)
        }
        6 => {
            // Already 6 fields, return as-is
            schedule.to_string()
        }
        _ => {
            // Invalid, but return as-is (validation will catch it)
            schedule.to_string()
        }
    }
}

/// Calculate the next execution timestamp from a cron schedule
///
/// # Arguments
/// * `schedule` - Cron expression (e.g., "0 10 * * *" for daily at 10:00 AM, or "0 0 10 * * *" with seconds)
///
/// # Returns
/// Next execution timestamp, or None if schedule is invalid or empty
pub fn calculate_next_execution(schedule: Option<&str>) -> Option<DateTime<Utc>> {
    let schedule_str = schedule?;
    let schedule_str = schedule_str.trim();
    if schedule_str.is_empty() {
        return None;
    }

    // Normalize to 6 fields (cron crate 0.15 requires 6 fields)
    let normalized = normalize_cron_schedule(schedule_str);

    // Parse cron expression
    let cron_schedule = match cron::Schedule::from_str(&normalized) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Invalid cron schedule '{}' (normalized: '{}'): {}", schedule_str, normalized, e);
            return None;
        }
    };

    // Get the next execution time from now
    let _now = Utc::now();
    cron_schedule.upcoming(Utc).next().map(|dt| dt.with_timezone(&Utc))
}

