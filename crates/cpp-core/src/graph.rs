//! Context Graph — typed relationship graph between context objects.

use serde::{Deserialize, Serialize};

/// A typed edge in the context graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub edge_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

/// The context graph returned alongside a ContextBundle.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<GraphEdge>,
    pub cycle_detected: bool,
}

impl ContextGraph {
    pub fn new() -> Self { Self::default() }
    pub fn add_node(&mut self, id: impl Into<String>) {
        let id = id.into();
        if !self.nodes.contains(&id) { self.nodes.push(id); }
    }
    pub fn add_edge(&mut self, source: impl Into<String>, target: impl Into<String>, edge_type: impl Into<String>) {
        self.edges.push(GraphEdge { source: source.into(), target: target.into(), edge_type: edge_type.into(), weight: None });
    }
    pub fn is_empty(&self) -> bool { self.nodes.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_graph_construction() {
        let mut graph = ContextGraph::new();
        graph.add_node("ctx_1");
        graph.add_node("ctx_2");
        graph.add_node("ctx_1"); // duplicate
        graph.add_edge("ctx_1", "ctx_2", "contains");
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }
    #[test]
    fn test_graph_serialization() {
        let mut graph = ContextGraph::new();
        graph.add_node("ctx_a");
        graph.add_edge("ctx_a", "ctx_b", "references");
        let json = serde_json::to_string(&graph).unwrap();
        assert!(json.contains("edgeType"));
        let parsed: ContextGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.edges[0].edge_type, "references");
    }
}
