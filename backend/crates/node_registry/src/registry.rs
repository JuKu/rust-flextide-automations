use std::collections::HashMap;
use flextide_sdk::{NodeDefinition, NodeGroup};

/// Registry for managing node groups and their associated node definitions
#[derive(Debug, Clone)]
pub struct NodeRegistry {
    /// List of all registered node groups
    groups: Vec<NodeGroup>,
    /// Map from group name to list of node definitions in that group
    nodes_by_group: HashMap<String, Vec<NodeDefinition>>,
}

impl NodeRegistry {
    /// Create a new empty node registry
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            nodes_by_group: HashMap::new(),
        }
    }

    /// Register a new node group
    pub fn register_group(&mut self, group: NodeGroup) {
        let group_name = group.name.clone();
        if !self.groups.iter().any(|g| g.name == group_name) {
            self.groups.push(group);
            self.nodes_by_group.insert(group_name, Vec::new());
        }
    }

    /// Remove a node group by name
    /// Returns true if the group was found and removed, false otherwise
    pub fn remove_group(&mut self, group_name: &str) -> bool {
        if let Some(pos) = self.groups.iter().position(|g| g.name == group_name) {
            self.groups.remove(pos);
            self.nodes_by_group.remove(group_name);
            true
        } else {
            false
        }
    }

    /// Register a new node definition
    /// The node's group must already be registered
    pub fn register_node(&mut self, node: NodeDefinition) -> Result<(), String> {
        let group_name = &node.group;
        if let Some(nodes) = self.nodes_by_group.get_mut(group_name) {
            // Check if node with same name already exists
            if nodes.iter().any(|n| n.name == node.name) {
                return Err(format!("Node '{}' already exists in group '{}'", node.name, group_name));
            }
            nodes.push(node);
            Ok(())
        } else {
            Err(format!("Group '{}' not found. Register the group first.", group_name))
        }
    }

    /// Remove a node by name from its group
    /// Returns true if the node was found and removed, false otherwise
    pub fn remove_node(&mut self, node_name: &str) -> bool {
        for nodes in self.nodes_by_group.values_mut() {
            if let Some(pos) = nodes.iter().position(|n| n.name == node_name) {
                nodes.remove(pos);
                return true;
            }
        }
        false
    }

    /// Get all node groups with their associated nodes
    pub fn list_groups_with_nodes(&self) -> Vec<(NodeGroup, Vec<NodeDefinition>)> {
        self.groups
            .iter()
            .map(|group| {
                let nodes = self.nodes_by_group
                    .get(&group.name)
                    .cloned()
                    .unwrap_or_default();
                (group.clone(), nodes)
            })
            .collect()
    }

    /// Get all node groups
    pub fn list_groups(&self) -> &[NodeGroup] {
        &self.groups
    }

    /// Get all nodes for a specific group by group name
    pub fn get_nodes_for_group(&self, group_name: &str) -> Option<&[NodeDefinition]> {
        self.nodes_by_group.get(group_name).map(|v| v.as_slice())
    }

    /// Get a node definition by name
    pub fn get_node(&self, node_name: &str) -> Option<&NodeDefinition> {
        for nodes in self.nodes_by_group.values() {
            if let Some(node) = nodes.iter().find(|n| n.name == node_name) {
                return Some(node);
            }
        }
        None
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

