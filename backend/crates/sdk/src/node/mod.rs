use serde::{Deserialize, Serialize};

/// Pin type for node inputs, outputs, and configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PinType {
    Exec,
    String,
    Number,
    Boolean,
    Json,
    Any,
    Custom,
}

/// Input pin definition for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputPin {
    /// Unique identifier for this input pin
    pub name: String,
    /// Display title (e.g., "First Value")
    pub title: String,
    /// Description of what this input accepts
    pub description: String,
    /// Type of data this pin accepts
    pub pin_type: PinType,
    /// Custom type value (only used when pin_type is Custom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_type: Option<String>,
}

/// Output pin definition for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputPin {
    /// Unique identifier for this output pin
    pub name: String,
    /// Display title (e.g., "Result")
    pub title: String,
    /// Description of what this output provides
    pub description: String,
    /// Type of data this pin produces
    pub pin_type: PinType,
    /// Custom type value (only used when pin_type is Custom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_type: Option<String>,
}

/// Configuration option for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOption {
    /// Unique identifier for this config option
    pub name: String,
    /// Display title (e.g., "Case Sensitive")
    pub title: String,
    /// Description of what this option does
    pub description: String,
    /// Type of the configuration value
    pub option_type: PinType,
    /// Custom type value (only used when option_type is Custom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_type: Option<String>,
    /// Whether this option is required
    pub required: bool,
}

/// Node group definition (e.g., "Logic", "HTTP", "Database")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroup {
    /// Unique identifier for this group (e.g., "logic", "http", "database")
    pub name: String,
    /// Display title (e.g., "Logic", "HTTP", "Database")
    pub title: String,
    /// Description of what nodes in this group do
    pub description: String,
}

/// Complete node definition for the node catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    /// Unique identifier for this node (e.g., "or", "http-request")
    pub name: String,
    /// Display title (e.g., "OR", "HTTP Request")
    pub title: String,
    /// Description of what this node does
    pub description: String,
    /// The node group this node belongs to
    pub group: String,
    /// List of input pins (data inputs on the left side)
    pub inputs: Vec<InputPin>,
    /// List of output pins (data outputs on the right side)
    pub outputs: Vec<OutputPin>,
    /// Configuration options (shown in bottom config section)
    pub config: Vec<ConfigOption>,
}

