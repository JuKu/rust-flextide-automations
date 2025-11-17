use serde::{Deserialize, Serialize};
use serde_json::Value;

/// ABI version constant
pub const ABI_VERSION: &str = "1.0";

/// Node execution request - what gets sent to any node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionRequest {
    /// ABI version for compatibility checking
    #[serde(default = "default_abi_version")]
    pub abi_version: String,

    /// Input pin values keyed by pin name
    /// Exec pins are represented as boolean (true = triggered)
    pub input: Value, // Map<String, Value>

    /// Configuration values keyed by config option name
    pub config: Value, // Map<String, Value>

    /// Execution context (optional metadata)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ExecutionContext>,
}

fn default_abi_version() -> String {
    ABI_VERSION.to_string()
}

/// Execution context metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub workflow_id: String,
    pub run_id: String,
    pub node_id: String,
    pub execution_id: String,
}

/// Node execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionResponse {
    /// Output pin values keyed by pin name
    pub output: Value, // Map<String, Value>

    /// Whether exec-out should fire (default: true if no error)
    #[serde(default = "default_true")]
    pub exec_out: bool,

    /// Error if execution failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<NodeError>,
}

fn default_true() -> bool {
    true
}

/// Node execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeError {
    pub message: String,
    pub code: Option<String>,
    pub details: Option<Value>,
}

/// Helper for building execution requests from pin values
#[derive(Debug, Default)]
pub struct ExecutionRequestBuilder {
    input: Value,
    config: Value,
    context: Option<ExecutionContext>,
}

impl ExecutionRequestBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            input: Value::Object(serde_json::Map::new()),
            config: Value::Object(serde_json::Map::new()),
            context: None,
        }
    }

    /// Add an input pin value
    pub fn with_input(mut self, pin_name: impl Into<String>, value: Value) -> Self {
        if let Value::Object(ref mut map) = self.input {
            map.insert(pin_name.into(), value);
        }
        self
    }

    /// Add multiple input pin values
    pub fn with_inputs(mut self, inputs: impl IntoIterator<Item = (String, Value)>) -> Self {
        if let Value::Object(ref mut map) = self.input {
            for (key, value) in inputs {
                map.insert(key, value);
            }
        }
        self
    }

    /// Add a config option value
    pub fn with_config(mut self, option_name: impl Into<String>, value: Value) -> Self {
        if let Value::Object(ref mut map) = self.config {
            map.insert(option_name.into(), value);
        }
        self
    }

    /// Add multiple config option values
    pub fn with_configs(mut self, configs: impl IntoIterator<Item = (String, Value)>) -> Self {
        if let Value::Object(ref mut map) = self.config {
            for (key, value) in configs {
                map.insert(key, value);
            }
        }
        self
    }

    /// Set execution context
    pub fn with_context(mut self, context: ExecutionContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Build the execution request
    pub fn build(self) -> NodeExecutionRequest {
        NodeExecutionRequest {
            abi_version: ABI_VERSION.to_string(),
            input: self.input,
            config: self.config,
            context: self.context,
        }
    }
}

/// Helper for building execution responses
#[derive(Debug, Default)]
pub struct ExecutionResponseBuilder {
    output: Value,
    exec_out: bool,
    error: Option<NodeError>,
}

impl ExecutionResponseBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            output: Value::Object(serde_json::Map::new()),
            exec_out: true,
            error: None,
        }
    }

    /// Add an output pin value
    pub fn with_output(mut self, pin_name: impl Into<String>, value: Value) -> Self {
        if let Value::Object(ref mut map) = self.output {
            map.insert(pin_name.into(), value);
        }
        self
    }

    /// Add multiple output pin values
    pub fn with_outputs(mut self, outputs: impl IntoIterator<Item = (String, Value)>) -> Self {
        if let Value::Object(ref mut map) = self.output {
            for (key, value) in outputs {
                map.insert(key, value);
            }
        }
        self
    }

    /// Set exec-out flag (whether execution should continue)
    pub fn with_exec_out(mut self, exec_out: bool) -> Self {
        self.exec_out = exec_out;
        self
    }

    /// Set error
    pub fn with_error(mut self, message: impl Into<String>) -> Self {
        self.error = Some(NodeError {
            message: message.into(),
            code: None,
            details: None,
        });
        self.exec_out = false;
        self
    }

    /// Set error with code
    pub fn with_error_code(
        mut self,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        self.error = Some(NodeError {
            message: message.into(),
            code: Some(code.into()),
            details: None,
        });
        self.exec_out = false;
        self
    }

    /// Set error with full details
    pub fn with_error_full(
        mut self,
        message: impl Into<String>,
        code: Option<String>,
        details: Option<Value>,
    ) -> Self {
        self.error = Some(NodeError {
            message: message.into(),
            code,
            details,
        });
        self.exec_out = false;
        self
    }

    /// Build the execution response
    pub fn build(self) -> NodeExecutionResponse {
        NodeExecutionResponse {
            output: self.output,
            exec_out: self.exec_out,
            error: self.error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_request_builder() {
        let request = ExecutionRequestBuilder::new()
            .with_input("value1", Value::String("test".to_string()))
            .with_input("value2", Value::Number(42.into()))
            .with_config("option1", Value::Bool(true))
            .build();

        assert_eq!(request.abi_version, ABI_VERSION);
        assert!(request.input.get("value1").is_some());
        assert!(request.config.get("option1").is_some());
    }

    #[test]
    fn test_execution_response_builder() {
        let response = ExecutionResponseBuilder::new()
            .with_output("result", Value::String("success".to_string()))
            .with_exec_out(true)
            .build();

        assert!(response.output.get("result").is_some());
        assert!(response.exec_out);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_execution_response_with_error() {
        let response = ExecutionResponseBuilder::new()
            .with_error("Something went wrong")
            .build();

        assert!(!response.exec_out);
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().message, "Something went wrong");
    }
}

