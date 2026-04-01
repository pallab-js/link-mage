use std::sync::{Arc, RwLock};
use crate::graph::Graph;
use crate::pipeline::PipelineRegistry;
use crate::plugin_api::PluginRegistry;
use crate::transforms::TransformRegistry;
use crate::entities::EntityRegistry;

/// Central application state shared across all Tauri commands.
///
/// - `graph`   — `RwLock` enables concurrent reads by non-mutating commands.
/// - `registry` — built-in transforms; read-only after construction.
/// - `plugins` — dynamically loaded external plugin transforms; write-locked
///   only during `load_plugins_dir`.
/// - `pipeline` — pipeline registry for transform chaining.
#[derive(Clone)]
pub struct AppState {
    pub graph:    Arc<RwLock<Graph>>,
    pub registry: Arc<TransformRegistry>,
    pub plugins:  Arc<RwLock<PluginRegistry>>,
    pub entities: Arc<EntityRegistry>,
    pub remote_transforms: Arc<RwLock<Vec<crate::transforms::RemoteTransform>>>,
    pub pipeline: Arc<RwLock<PipelineRegistry>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            graph:    Arc::new(RwLock::new(Graph::new())),
            registry: Arc::new(TransformRegistry::new()),
            plugins:  Arc::new(RwLock::new(PluginRegistry::new())),
            entities: Arc::new(EntityRegistry::new()),
            remote_transforms: Arc::new(RwLock::new(Vec::new())),
            pipeline: Arc::new(RwLock::new(PipelineRegistry::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
