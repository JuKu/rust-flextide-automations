use serde::{Deserialize, Serialize};

/// Plugin definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    /// Unique name of the plugin
    pub name: String,
}

