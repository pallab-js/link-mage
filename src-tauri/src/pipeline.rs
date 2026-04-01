//! Transform Chaining Pipelines
//!
//! Allows users to define and execute multi-step transform sequences.
//! Each pipeline consists of ordered steps that execute sequentially,
//! with each step's output potentially feeding into the next step.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::graph::{Node, Edge};
use crate::transforms::Transform;

/// Pipeline step configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Unique step identifier
    pub id: String,
    /// Name of the transform to execute
    pub transform_name: String,
    /// Expected input node type (empty means any type)
    pub input_type: String,
    /// Description of what this step does
    pub description: String,
    /// Timeout in seconds (0 means use default)
    pub timeout: u64,
    /// Whether to continue on error
    pub continue_on_error: bool,
    /// Maximum number of results to process (0 means unlimited)
    pub max_results: usize,
}

/// Pipeline execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PipelineStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for PipelineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineStatus::Idle => write!(f, "idle"),
            PipelineStatus::Running => write!(f, "running"),
            PipelineStatus::Paused => write!(f, "paused"),
            PipelineStatus::Completed => write!(f, "completed"),
            PipelineStatus::Failed => write!(f, "failed"),
            PipelineStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub transform_name: String,
    pub status: String, // pending, running, success, failed, skipped
    pub nodes_added: usize,
    pub edges_added: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub duration_ms: u64,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

/// Pipeline execution log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecution {
    pub id: String,
    pub pipeline_id: String,
    pub pipeline_name: String,
    pub seed_node_id: String,
    pub seed_node_type: String,
    pub status: String,
    pub steps: Vec<StepResult>,
    pub total_nodes_added: usize,
    pub total_edges_added: usize,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Pipeline definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Unique identifier
    pub id: String,
    /// Pipeline name
    pub name: String,
    /// Pipeline description
    pub description: String,
    /// Pipeline steps in execution order
    pub steps: Vec<PipelineStep>,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last modified timestamp (ISO 8601)
    pub updated_at: String,
    /// Number of times pipeline has been executed
    pub execution_count: usize,
    /// Whether pipeline is enabled
    pub enabled: bool,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Pipeline {
    /// Create a new pipeline
    pub fn new(name: &str, description: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            steps: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
            execution_count: 0,
            enabled: true,
            tags: Vec::new(),
        }
    }

    /// Add a step to the pipeline
    pub fn add_step(&mut self, step: PipelineStep) {
        self.steps.push(step);
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Validate pipeline configuration
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push("Pipeline name is required".to_string());
        }

        if self.steps.is_empty() {
            errors.push("Pipeline must have at least one step".to_string());
        } else {
            for (index, step) in self.steps.iter().enumerate() {
                if step.transform_name.trim().is_empty() {
                    errors.push(format!("Step {}: Transform name is required", index + 1));
                }
                if step.timeout > 300 {
                    errors.push(format!(
                        "Step {}: Timeout must be between 0 and 300 seconds",
                        index + 1
                    ));
                }
            }
        }

        errors
    }

    /// Get estimated execution time in seconds
    pub fn estimated_duration(&self) -> u64 {
        self.steps
            .iter()
            .map(|s| if s.timeout > 0 { s.timeout } else { 30 })
            .sum()
    }
}

/// Pipeline executor - runs a pipeline against a seed node
pub struct PipelineExecutor<'a> {
    pipeline: &'a Pipeline,
    seed_node: &'a Node,
    execution: PipelineExecution,
    cancel_flag: Arc<AtomicBool>,
    nodes_added: Arc<AtomicUsize>,
    edges_added: Arc<AtomicUsize>,
}

impl<'a> PipelineExecutor<'a> {
    pub fn new(pipeline: &'a Pipeline, seed_node: &'a Node) -> Self {
        let now = Utc::now().to_rfc3339();
        let execution = PipelineExecution {
            id: uuid::Uuid::new_v4().to_string(),
            pipeline_id: pipeline.id.clone(),
            pipeline_name: pipeline.name.clone(),
            seed_node_id: seed_node.id.clone(),
            seed_node_type: seed_node.node_type.clone(),
            status: "running".to_string(),
            steps: pipeline
                .steps
                .iter()
                .map(|s| StepResult {
                    step_id: s.id.clone(),
                    transform_name: s.transform_name.clone(),
                    status: "pending".to_string(),
                    nodes_added: 0,
                    edges_added: 0,
                    error: None,
                    duration_ms: 0,
                    started_at: now.clone(),
                    completed_at: None,
                })
                .collect(),
            total_nodes_added: 0,
            total_edges_added: 0,
            started_at: now,
            completed_at: None,
            error: None,
        };

        Self {
            pipeline,
            seed_node,
            execution,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            nodes_added: Arc::new(AtomicUsize::new(0)),
            edges_added: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get a handle to cancel execution
    pub fn get_cancel_handle(&self) -> Arc<AtomicBool> {
        self.cancel_flag.clone()
    }

    /// Get current execution state
    pub fn get_execution(&self) -> &PipelineExecution {
        &self.execution
    }

    /// Execute the pipeline
    pub fn execute(
        &mut self,
        transforms: &[&dyn Transform],
    ) -> Result<(Vec<Node>, Vec<Edge>), String> {
        self.execution.status = "running".to_string();

        let mut all_nodes = Vec::new();
        let mut all_edges = Vec::new();
        let mut current_nodes = vec![self.seed_node.clone()];

        for (step_index, step) in self.pipeline.steps.iter().enumerate() {
            // Check for cancellation
            if self.cancel_flag.load(Ordering::Relaxed) {
                self.execution.status = "cancelled".to_string();
                self.execution.completed_at = Some(Utc::now().to_rfc3339());
                self.execution.error = Some("Pipeline execution cancelled".to_string());
                break;
            }

            // Update step status to running
            self.execution.steps[step_index].status = "running".to_string();
            self.execution.steps[step_index].started_at = Utc::now().to_rfc3339();
            let step_start = std::time::Instant::now();

            // Find matching transform
            let transform = transforms.iter().find(|t| t.name() == step.transform_name);

            if transform.is_none() {
                let error = format!("Transform '{}' not found", step.transform_name);
                self.execution.steps[step_index].status = "failed".to_string();
                self.execution.steps[step_index].error = Some(error.clone());
                self.execution.steps[step_index].completed_at = Some(Utc::now().to_rfc3339());
                self.execution.steps[step_index].duration_ms = step_start.elapsed().as_millis() as u64;

                if !step.continue_on_error {
                    self.execution.status = "failed".to_string();
                    self.execution.error = Some(error.clone());
                    self.execution.completed_at = Some(Utc::now().to_rfc3339());
                    return Err(error);
                }
                continue;
            }

            let transform = transform.unwrap();

            // Execute transform for each current node
            let mut step_nodes = Vec::new();
            let mut step_edges = Vec::new();
            let mut step_error: Option<String> = None;

            for node in &current_nodes {
                // Check node type compatibility
                if !step.input_type.is_empty() && node.node_type != step.input_type {
                    continue;
                }

                // Check max results limit
                if step.max_results > 0
                    && step_nodes.len() >= step.max_results
                {
                    break;
                }

                // Check timeout (simplified - actual timeout would require async)
                let _timeout = if step.timeout > 0 {
                    Duration::from_secs(step.timeout)
                } else {
                    Duration::from_secs(30)
                };

                // Execute transform (simplified - in production would use async with timeout)
                match transform.run(node) {
                    Ok((mut nodes, mut edges)) => {
                        // Apply max results limit
                        if step.max_results > 0 {
                            let remaining = step.max_results - step_nodes.len();
                            if nodes.len() > remaining {
                                nodes.truncate(remaining);
                                edges.truncate(remaining);
                            }
                        }

                        step_nodes.extend(nodes);
                        step_edges.extend(edges);
                    }
                    Err(e) => {
                        step_error = Some(format!(
                            "Transform failed for node {}: {}",
                            node.id, e
                        ));
                        if !step.continue_on_error {
                            break;
                        }
                    }
                }
            }

            let step_duration = step_start.elapsed().as_millis() as u64;

            // Update step result
            self.execution.steps[step_index].status = if step_error.is_some() && !step.continue_on_error {
                "failed".to_string()
            } else {
                "success".to_string()
            };
            self.execution.steps[step_index].nodes_added = step_nodes.len();
            self.execution.steps[step_index].edges_added = step_edges.len();
            self.execution.steps[step_index].error = step_error;
            self.execution.steps[step_index].duration_ms = step_duration;
            self.execution.steps[step_index].completed_at = Some(Utc::now().to_rfc3339());

            // Update totals
            self.execution.total_nodes_added += step_nodes.len();
            self.execution.total_edges_added += step_edges.len();
            self.nodes_added.fetch_add(step_nodes.len(), Ordering::Relaxed);
            self.edges_added.fetch_add(step_edges.len(), Ordering::Relaxed);

            all_nodes.extend(step_nodes);
            all_edges.extend(step_edges);

            // Prepare nodes for next step
            current_nodes = all_nodes.iter().map(|n| n.clone()).collect();
        }

        // Mark execution as completed
        if self.execution.status == "running" {
            self.execution.status = "completed".to_string();
        }
        self.execution.completed_at = Some(Utc::now().to_rfc3339());

        Ok((all_nodes, all_edges))
    }

    /// Get total nodes added
    pub fn nodes_added(&self) -> usize {
        self.nodes_added.load(Ordering::Relaxed)
    }

    /// Get total edges added
    pub fn edges_added(&self) -> usize {
        self.edges_added.load(Ordering::Relaxed)
    }
}

/// Pipeline registry - manages available pipelines
pub struct PipelineRegistry {
    pipelines: Vec<Pipeline>,
}

impl PipelineRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            pipelines: Vec::new(),
        };

        // Add built-in pipeline templates
        registry.add_builtin_pipelines();
        registry
    }

    fn add_builtin_pipelines(&mut self) {
        // Domain Enrichment Pipeline
        let mut domain_enrichment = Pipeline::new(
            "Domain Enrichment",
            "Comprehensive domain intelligence gathering",
        );
        domain_enrichment.add_step(PipelineStep {
            id: "step1".to_string(),
            transform_name: "DNS Lookup".to_string(),
            input_type: "Domain".to_string(),
            description: "Resolve to IP addresses".to_string(),
            timeout: 30,
            continue_on_error: false,
            max_results: 0,
        });
        domain_enrichment.add_step(PipelineStep {
            id: "step2".to_string(),
            transform_name: "GeoIP Lookup".to_string(),
            input_type: "IP".to_string(),
            description: "Geolocate IP addresses".to_string(),
            timeout: 30,
            continue_on_error: true,
            max_results: 0,
        });
        domain_enrichment.add_step(PipelineStep {
            id: "step3".to_string(),
            transform_name: "WHOIS Lookup".to_string(),
            input_type: "Domain".to_string(),
            description: "Get domain registration info".to_string(),
            timeout: 30,
            continue_on_error: true,
            max_results: 0,
        });
        domain_enrichment.tags = vec!["dns".to_string(), "domain".to_string(), "enrichment".to_string()];
        self.pipelines.push(domain_enrichment);

        // IP Reconnaissance Pipeline
        let mut ip_recon = Pipeline::new(
            "IP Reconnaissance",
            "Gather intelligence on IP addresses",
        );
        ip_recon.add_step(PipelineStep {
            id: "step1".to_string(),
            transform_name: "GeoIP Lookup".to_string(),
            input_type: "IP".to_string(),
            description: "Geolocate IP".to_string(),
            timeout: 30,
            continue_on_error: false,
            max_results: 0,
        });
        ip_recon.add_step(PipelineStep {
            id: "step2".to_string(),
            transform_name: "Reverse DNS".to_string(),
            input_type: "IP".to_string(),
            description: "Find hostnames".to_string(),
            timeout: 30,
            continue_on_error: true,
            max_results: 0,
        });
        ip_recon.tags = vec!["ip".to_string(), "recon".to_string(), "security".to_string()];
        self.pipelines.push(ip_recon);
    }

    /// Get all pipelines
    pub fn list(&self) -> &[Pipeline] {
        &self.pipelines
    }

    /// Get pipeline by ID
    pub fn get(&self, id: &str) -> Option<&Pipeline> {
        self.pipelines.iter().find(|p| p.id == id)
    }

    /// Get pipeline by name
    pub fn get_by_name(&self, name: &str) -> Option<&Pipeline> {
        self.pipelines.iter().find(|p| p.name == name)
    }

    /// Add a pipeline
    pub fn add(&mut self, pipeline: Pipeline) {
        self.pipelines.push(pipeline);
    }

    /// Remove a pipeline
    pub fn remove(&mut self, id: &str) -> bool {
        let pos = self.pipelines.iter().position(|p| p.id == id);
        if let Some(idx) = pos {
            self.pipelines.remove(idx);
            true
        } else {
            false
        }
    }

    /// Get pipelines by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&Pipeline> {
        self.pipelines
            .iter()
            .filter(|p| p.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get enabled pipelines
    pub fn enabled(&self) -> Vec<&Pipeline> {
        self.pipelines.iter().filter(|p| p.enabled).collect()
    }
}

impl Default for PipelineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let mut pipeline = Pipeline::new("Test Pipeline", "Test Description");
        assert_eq!(pipeline.name, "Test Pipeline");
        assert!(pipeline.enabled);
        assert!(pipeline.steps.is_empty());
    }

    #[test]
    fn test_pipeline_validation() {
        let pipeline = Pipeline::new("", "Description");
        let errors = pipeline.validate();
        assert!(errors.contains(&"Pipeline name is required".to_string()));
    }

    #[test]
    fn test_pipeline_registry() {
        let registry = PipelineRegistry::new();
        assert!(!registry.list().is_empty());
        
        let domain_pipeline = registry.get_by_name("Domain Enrichment");
        assert!(domain_pipeline.is_some());
    }
}
