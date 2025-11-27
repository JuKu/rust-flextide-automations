//! Organizational Settings Module
//!
//! Provides functionality for managing and retrieving organizational settings.

mod database;

pub use database::{get_organizational_setting_value, SettingsDatabaseError};

