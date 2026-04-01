use std::collections::HashMap;
use std::path::Path;
use serde::Serialize;
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;
use crate::entities::EntityDefinition;
use crate::graph::{Edge, Graph, Node};
use crate::transforms::{RemoteTransform, Transform};
use crate::keychain;
use crate::plugin_api::{scan_plugin_dir, PluginLoadResult};
use crate::state::AppState;

// ── Error type ────────────────────────────────────────────────────────────────

/// A transparent string error that serialises cleanly to the frontend.
type CmdResult<T> = Result<T, String>;

fn lock_err() -> String {
    "Graph RwLock poisoned".to_string()
}

// ── Lock helpers ──────────────────────────────────────────────────────────────

/// Acquire an **exclusive write** lock and run `f` against the graph.
fn with_graph<F, T>(state: &State<AppState>, f: F) -> CmdResult<T>
where
    F: FnOnce(&mut Graph) -> CmdResult<T>,
{
    let mut graph = state.graph.write().map_err(|_| lock_err())?;
    f(&mut graph)
}

/// Acquire a **shared read** lock and run `f` against the graph.
fn with_graph_ro<F, T>(state: &State<AppState>, f: F) -> CmdResult<T>
where
    F: FnOnce(&Graph) -> CmdResult<T>,
{
    let graph = state.graph.read().map_err(|_| lock_err())?;
    f(&graph)
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Return a list of all defined entity types and their metadata.
#[tauri::command]
pub fn list_entities(state: State<AppState>) -> Vec<EntityDefinition> {
    state.entities.all()
}

/// Return a full serialised snapshot of the graph.
#[tauri::command]
pub fn get_graph(state: State<AppState>) -> CmdResult<Graph> {
    with_graph_ro(&state, |g| Ok(g.clone()))
}

/// Add a node. Returns the id of the inserted node.
///
/// The caller may pre-fill `node.id`; if it is empty a new UUID is generated.
/// All other fields (`node_type`, `properties`) are preserved as-is.
#[tauri::command]
pub fn add_node(state: State<AppState>, mut node: Node) -> CmdResult<String> {
    if node.id.is_empty() {
        node.id = Uuid::new_v4().to_string();
    }
    with_graph(&state, |g| {
        let id = node.id.clone();
        if g.add_node(node) {
            Ok(id)
        } else {
            Err(format!("Node with id '{}' already exists", id))
        }
    })
}

/// Overwrite the `properties` map of an existing node.
#[tauri::command]
pub fn update_node(
    state: State<AppState>,
    id: String,
    properties: HashMap<String, String>,
) -> CmdResult<()> {
    with_graph(&state, |g| {
        let node = g
            .find_node_mut(&id)
            .ok_or_else(|| format!("Node '{}' not found", id))?;
        node.properties = properties;
        Ok(())
    })
}

/// Remove a node and all edges that reference it.
#[tauri::command]
pub fn remove_node(state: State<AppState>, id: String) -> CmdResult<()> {
    with_graph(&state, |g| {
        g.remove_node(&id)
            .map(|_| ())
            .ok_or_else(|| format!("Node '{}' not found", id))
    })
}

/// Add an edge. Returns the id of the inserted edge.
///
/// The caller may pre-fill `edge.id`; if it is empty a new UUID is generated.
#[tauri::command]
pub fn add_edge(state: State<AppState>, mut edge: Edge) -> CmdResult<String> {
    if edge.id.is_empty() {
        edge = Edge::new(
            edge.source.clone(),
            edge.target.clone(),
            edge.label.clone(),
        );
    }
    with_graph(&state, |g| {
        let id = edge.id.clone();
        if g.add_edge(edge) {
            Ok(id)
        } else {
            Err("Edge could not be added: source/target missing or id already exists".to_string())
        }
    })
}

/// Remove an edge by id.
#[tauri::command]
pub fn remove_edge(state: State<AppState>, id: String) -> CmdResult<()> {
    with_graph(&state, |g| {
        g.remove_edge(&id)
            .map(|_| ())
            .ok_or_else(|| format!("Edge '{}' not found", id))
    })
}

/// Replace the current graph with an empty one.
#[tauri::command]
pub fn clear_graph(state: State<AppState>) -> CmdResult<()> {
    with_graph(&state, |g| { *g = Graph::new(); Ok(()) })
}

/// Save the current graph to a path that is already known (no dialog).
/// Used by Ctrl+S when a file has been saved/loaded previously.
#[tauri::command]
pub fn save_graph_to_path(state: State<AppState>, path: String) -> CmdResult<()> {
    with_graph_ro(&state, |g| {
        g.save_graph(Path::new(&path))
            .map_err(|e| format!("Save failed: {e}"))
    })
}

// ── File dialog commands ───────────────────────────────────────────────────────

/// Carries both the loaded graph and the path it came from.
#[derive(Serialize)]
pub struct LoadResult {
    pub graph: Graph,
    pub path: String,
}

/// Open a save-file dialog filtered to `.json`, then serialize the current
/// graph to the chosen path.
/// Returns the path that was saved to, or `None` if the user cancelled.
#[tauri::command]
pub fn save_graph_dialog(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> CmdResult<Option<String>> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("JSON Graph", &["json"])
        .set_file_name("graph.json")
        .blocking_save_file();

    match file_path {
        None => Ok(None), // user cancelled
        Some(fp) => {
            let path = fp
                .into_path()
                .map_err(|e| format!("Invalid path: {e}"))?;

            // Read lock: multiple save calls can run concurrently.
            with_graph_ro(&state, |g| {
                g.save_graph(&path).map_err(|e| format!("Save failed: {e}"))
            })?;

            Ok(Some(path.to_string_lossy().into_owned()))
        }
    }
}

/// Open an open-file dialog filtered to `.json`, load the graph from the
/// chosen path and replace the current in-memory graph.
/// Returns `{ graph, path }` on success, or `None` if the user cancelled.
#[tauri::command]
pub fn load_graph_dialog(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> CmdResult<Option<LoadResult>> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("JSON Graph", &["json"])
        .blocking_pick_file();

    match file_path {
        None => Ok(None), // user cancelled
        Some(fp) => {
            let path = fp
                .into_path()
                .map_err(|e| format!("Invalid path: {e}"))?;

            let loaded = Graph::load_graph(&path)
                .map_err(|e| format!("Load failed: {e}"))?;

            let path_str = path.to_string_lossy().into_owned();
            with_graph(&state, |g| { *g = loaded.clone(); Ok(()) })?;

            Ok(Some(LoadResult {
                graph: loaded,
                path: path_str,
            }))
        }
    }
}

// ── CSV export command ────────────────────────────────────────────────────────

/// Open a folder-picker dialog, look for `nodes.csv` + `edges.csv` inside,
/// parse them with [`Graph::import_csv`], and replace the current graph.
///
/// Returns `Some(graph)` on success, `None` if the user cancelled.
#[tauri::command]
pub fn import_csv_dialog(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> CmdResult<Option<Graph>> {
    let folder = app
        .dialog()
        .file()
        .set_title("Import CSV — choose folder containing nodes.csv and edges.csv")
        .blocking_pick_folder();

    match folder {
        None => Ok(None),
        Some(fp) => {
            let dir = fp
                .into_path()
                .map_err(|e| format!("Invalid folder path: {e}"))?;

            let nodes_path = dir.join("nodes.csv");
            let edges_path = dir.join("edges.csv");

            if !nodes_path.exists() {
                return Err(format!(
                    "nodes.csv not found in {}",
                    dir.display()
                ));
            }
            if !edges_path.exists() {
                return Err(format!(
                    "edges.csv not found in {}",
                    dir.display()
                ));
            }

            let imported = Graph::import_csv(&nodes_path, &edges_path)
                .map_err(|e| format!("CSV import failed: {e}"))?;

            with_graph(&state, |g| { *g = imported.clone(); Ok(()) })?;

            Ok(Some(imported))
        }
    }
}

/// Open a save-file dialog filtered to `.graphml`, then write the graph as
/// GraphML 1.0 XML to the chosen path.
///
/// Returns `Some(file_path)` on success, `None` if the user cancelled.
#[tauri::command]
pub fn export_graphml_dialog(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> CmdResult<Option<String>> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("GraphML", &["graphml", "xml"])
        .set_file_name("graph.graphml")
        .blocking_save_file();

    match file_path {
        None => Ok(None),
        Some(fp) => {
            let path = fp
                .into_path()
                .map_err(|e| format!("Invalid path: {e}"))?;

            with_graph_ro(&state, |g| {
                g.export_graphml(&path)
                    .map_err(|e| format!("GraphML export failed: {e}"))
            })?;

            Ok(Some(path.to_string_lossy().into_owned()))
        }
    }
}

/// Open a folder-picker dialog, then write `nodes.csv` and `edges.csv` into
/// the chosen directory.
///
/// Returns `Some(folder_path)` on success, `None` if the user cancelled.
#[tauri::command]
pub fn export_csv_dialog(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> CmdResult<Option<String>> {
    let folder = app
        .dialog()
        .file()
        .set_title("Export CSV — choose destination folder")
        .blocking_pick_folder();

    match folder {
        None => Ok(None),
        Some(fp) => {
            let dir = fp
                .into_path()
                .map_err(|e| format!("Invalid folder path: {e}"))?;

            let nodes_path = dir.join("nodes.csv");
            let edges_path = dir.join("edges.csv");

            with_graph_ro(&state, |g| {
                g.export_csv(&nodes_path, &edges_path)
                    .map_err(|e| format!("CSV export failed: {e}"))
            })?;

            Ok(Some(dir.to_string_lossy().into_owned()))
        }
    }
}

// ── Transform commands ────────────────────────────────────────────────────────

/// Metadata about a single transform, serialised to the frontend.
///
/// `plugin_path` is `None` for built-in transforms and `Some(path)` for
/// transforms loaded from an external `.dylib` / `.so` / `.dll`.
#[derive(Serialize, Clone)]
pub struct TransformInfo {
    pub name:             String,
    pub description:      String,
    pub applicable_types: Vec<String>,
    /// Absolute path of the plugin file, or `None` for built-in transforms.
    pub plugin_path:      Option<String>,
}

/// What a successful `run_transform` call returns.
///
/// The frontend merges both slices directly into the store so it never needs
/// to re-fetch the whole graph after a transform.
#[derive(Serialize)]
pub struct TransformResult {
    /// Nodes that were created during this transform run.
    pub new_nodes: Vec<Node>,
    /// Edges that were created during this transform run.
    pub new_edges: Vec<Edge>,
}

/// List all transforms — both built-in and from loaded plugins.
///
/// The frontend calls this once on mount to build per-node-type context menus.
/// `plugin_path` distinguishes built-in (`null`) from external transforms.
fn save_remote_transforms(app: &tauri::AppHandle, remote: &[RemoteTransform]) -> CmdResult<()> {
    let base = app.path().app_data_dir().map_err(|e: tauri::Error| e.to_string())?;
    let path = base.join("remote_transforms.json");
    let json = serde_json::to_string_pretty(remote).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Add a new remote (HTTP) transform to the registry.
#[tauri::command]
pub fn add_remote_transform(
    app: tauri::AppHandle,
    state: State<AppState>,
    transform: RemoteTransform,
) -> CmdResult<()> {
    let mut remote = state.remote_transforms.write().map_err(|_| "Remote registry poisoned")?;
    // Check for name collision
    if remote.iter().any(|r| r.name == transform.name) {
        return Err(format!("Transform with name '{}' already exists", transform.name));
    }
    remote.push(transform);
    save_remote_transforms(&app, &remote)?;
    Ok(())
}

/// Remove a remote transform by name.
#[tauri::command]
pub fn remove_remote_transform(
    app: tauri::AppHandle,
    state: State<AppState>,
    name: String,
) -> CmdResult<()> {
    let mut remote = state.remote_transforms.write().map_err(|_| "Remote registry poisoned")?;
    let pos = remote.iter().position(|r| r.name == name)
        .ok_or_else(|| format!("Remote transform '{}' not found", name))?;
    remote.remove(pos);
    save_remote_transforms(&app, &remote)?;
    Ok(())
}

/// List all remote (HTTP) transforms.
#[tauri::command]
pub fn list_remote_transforms(state: State<AppState>) -> CmdResult<Vec<RemoteTransform>> {
    let remote = state.remote_transforms.read().map_err(|_| "Remote registry poisoned")?;
    Ok(remote.clone())
}

#[tauri::command]
pub fn list_transforms(state: State<AppState>) -> CmdResult<Vec<TransformInfo>> {
    let mut infos: Vec<TransformInfo> = state
        .registry
        .all()
        .iter()
        .map(|t| TransformInfo {
            name:             t.name(),
            description:      t.description(),
            applicable_types: t.applicable_types(),
            plugin_path:      None,
        })
        .collect();

    // Append plugin transforms.
    let plugins = state.plugins.read().map_err(|_| "Plugin registry poisoned")?;
    for p in plugins.all() {
        infos.push(TransformInfo {
            name:             p.meta.name.clone(),
            description:      p.meta.description.clone(),
            applicable_types: p.meta.applicable_types.clone(),
            plugin_path:      Some(p.meta.path.to_string_lossy().into_owned()),
        });
    }

    // Append remote transforms.
    let remote = state.remote_transforms.read().map_err(|_| "Remote registry poisoned")?;
    for r in remote.iter() {
        infos.push(TransformInfo {
            name:             r.name.clone(),
            description:      r.description.clone(),
            applicable_types: r.applicable_types.clone(),
            plugin_path:      Some(r.url.clone()),
        });
    }

    Ok(infos)
}

#[derive(Serialize)]
pub struct MachineResult {
    pub new_nodes: Vec<Node>,
    pub new_edges: Vec<Edge>,
}

/// Returns a list of all available automation "Machines".
#[tauri::command]
pub fn list_machines(state: State<'_, AppState>) -> Vec<crate::transforms::Machine> {
    state.registry.machines.machines.clone()
}

/// Runs a sequence of transforms ("Machine") on a single seed node.
///
/// This performs an iterative expansion: each transform in the machine is
/// executed, and the results are committed to the graph.
#[tauri::command]
pub async fn run_machine(
    state:          State<'_, AppState>,
    node_id:        String,
    machine_name:   String,
) -> CmdResult<MachineResult> {
    let app_state = (*state).clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_state;
        let machine = state.registry.machines.machines.iter()
            .find(|m| m.name == machine_name)
            .ok_or_else(|| format!("Machine '{}' not found", machine_name))?;

        let mut total_nodes = Vec::new();
        let mut total_edges = Vec::new();

        for transform_name in &machine.transforms {
            if let Some(transform) = state.registry.find(transform_name) {
                // Step 1: Resolve current state of seed node
                let seed = {
                    let graph = state.graph.read().map_err(|_| lock_err())?;
                    graph.find_node(&node_id).cloned()
                };

                let seed = match seed {
                    Some(s) => s,
                    None => {
                        log::warn!("[machine] seed node '{}' disappeared, stopping", node_id);
                        break;
                    }
                };

                // Step 2: Check applicability
                let types = transform.applicable_types();
                if !types.is_empty() && !types.contains(&seed.node_type) {
                    log::info!("[machine] skipping '{}' — not applicable to {}", transform_name, seed.node_type);
                    continue;
                }

                // Step 3: Run transform (no lock)
                if let Ok((new_nodes, new_edges)) = transform.run(&seed) {
                    // Step 4: Commit and track
                    {
                        let mut graph = state.graph.write().map_err(|_| lock_err())?;
                        for n in &new_nodes {
                            if graph.add_node(n.clone()) { total_nodes.push(n.clone()); }
                        }
                        for e in &new_edges {
                            if graph.add_edge(e.clone()) { total_edges.push(e.clone()); }
                        }
                    }
                }
            }
        }

        Ok(MachineResult { new_nodes: total_nodes, new_edges: total_edges })
    })
    .await
    .map_err(|e| format!("Machine thread panicked: {}", e))?
}

// ── Plugin management commands ────────────────────────────────────────────────

/// Scan a directory for plugin shared libraries, load every valid one, and
/// add them to the plugin registry.
///
/// If `dir` is omitted, the default plugin directory is used:
/// - macOS / Linux: `{app_data_dir}/plugins/`
/// - Created on first use if it does not exist.
///
/// Returns one `PluginLoadResult` entry per `.dylib` / `.so` / `.dll` file
/// found, indicating success or the reason for failure.  A non-empty result
/// with some `ok: false` entries does **not** cause the command to return an
/// `Err` — partial success is valid.
#[tauri::command]
pub fn load_plugins_dir(
    app:   tauri::AppHandle,
    state: State<AppState>,
    dir:   Option<String>,
) -> CmdResult<Vec<PluginLoadResult>> {
    use tauri::Manager;

    let plugin_dir = match dir {
        Some(d) => std::path::PathBuf::from(d),
        None    => {
            let base = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Could not resolve app data dir: {e}"))?;
            base.join("plugins")
        }
    };

    // Create the directory silently if it does not exist yet.
    if !plugin_dir.exists() {
        std::fs::create_dir_all(&plugin_dir)
            .map_err(|e| format!("Cannot create plugin dir '{}': {e}", plugin_dir.display()))?;
    }

    let (loaded, results) = scan_plugin_dir(&plugin_dir);

    let mut reg = state.plugins.write().map_err(|_| "Plugin registry poisoned")?;
    reg.extend(loaded);

    Ok(results)
}

/// Run a named transform against a node in the shared graph.
///
/// **Lock protocol** — minimises contention for large, slow transforms:
/// 1. Acquire a **read** lock, clone the graph snapshot + edge-id set, then
///    **release** the lock immediately.
/// 2. Run the transform on the local clone.  This step may involve blocking
///    network I/O (DNS, WHOIS, GeoIP) and holds **no lock at all**.
/// 3. Re-acquire a **write** lock only to commit the finished snapshot.
///
/// If the transform returns `Err` the real graph is left completely unchanged.
/// The frontend receives only the delta (new nodes + new edges) so it never
/// needs to re-fetch the whole graph after a transform.
#[tauri::command]
pub async fn run_transform(
    state:          State<'_, AppState>,
    node_id:        String,
    transform_name: String,
) -> CmdResult<TransformResult> {
    let app_state = (*state).clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_state;
        let t0 = std::time::Instant::now();

        // ── Step 1: Resolve seed node under read lock ────────────────────────
        let seed = {
            let graph = state.graph.read().map_err(|_| lock_err())?;
            graph.find_node(&node_id)
                .cloned()
                .ok_or_else(|| format!("Node '{node_id}' not found"))?
        };
        let t_clone = t0.elapsed();

        // ── Step 2: Dispatch and run with NO lock ─────────────────────────────
        let (new_nodes, new_edges_raw) = if let Some(transform) = state.registry.find(&transform_name) {
            transform.run(&seed)?
        } else {
            let plugins = state.plugins.read().map_err(|_| "Plugin registry poisoned")?;
            if let Some(plugin) = plugins.find(&transform_name) {
                plugin.run(&seed)?
            } else {
                let remote = state.remote_transforms.read().map_err(|_| "Remote registry poisoned")?;
                let rt = remote.iter().find(|r| r.name == transform_name)
                    .ok_or_else(|| format!("Unknown transform: '{transform_name}'"))?;
                rt.run(&seed)?
            }
        };
        let t_run = t0.elapsed();

        // ── Step 3: Commit under write lock ──────────────────────────────────
        let mut committed_nodes = Vec::new();
        let mut committed_edges = Vec::new();
        {
            let mut graph = state.graph.write().map_err(|_| lock_err())?;
            for node in new_nodes {
                if graph.add_node(node.clone()) {
                    committed_nodes.push(node);
                }
            }
            for edge in new_edges_raw {
                if graph.add_edge(edge.clone()) {
                    committed_edges.push(edge);
                }
            }
        }

        log::info!(
            "[transform] '{}' on '{}' — find {:?} | run {:?} | total {:?} \
             | +{} nodes +{} edges",
            transform_name, node_id, t_clone, t_run - t_clone, t0.elapsed(),
            committed_nodes.len(), committed_edges.len(),
        );

        Ok(TransformResult { new_nodes: committed_nodes, new_edges: committed_edges })
    })
    .await
    .map_err(|e| format!("Transform thread panicked: {}", e))?
}

/// Replaces a set of nodes with a single "Collection" node.
/// All edges incident to the original nodes are re-routed to the collection node.
#[tauri::command]
pub fn collapse_nodes(
    state:   State<'_, AppState>,
    ids:     Vec<String>,
    label:   String,
) -> CmdResult<Node> {
    let mut graph = state.graph.write().map_err(|_| lock_err())?;
    graph.collapse_nodes(&ids, &label).map_err(|e| e)
}

// ── Keychain / API-key commands ───────────────────────────────────────────────
//
// These commands expose the platform keychain to the frontend so the Settings
// panel can manage API keys without ever storing them in plain text on disk.
//
// Security model:
// - Keys are stored in the OS keychain (macOS Keychain, Linux Secret Service,
//   Windows Credential Manager) — encrypted at rest by the OS.
// - `get_api_key` returns the raw key value so the user can view/copy it
//   from their own Settings panel.  This is intentional: the key belongs to
//   the user and is already accessible via the OS keychain manager anyway.
// - The frontend never receives a key except when the user explicitly asks
//   for it via the Settings panel.

/// Metadata about a service whose API key can be stored.
#[derive(Serialize, Clone)]
pub struct ApiServiceInfo {
    /// Keychain account identifier (e.g. `"virustotal"`).
    pub id:          String,
    /// Human-readable display name (e.g. `"VirusTotal"`).
    pub name:        String,
    /// Short description shown below the name in the Settings panel.
    pub description: String,
    /// URL where the user can create/find their API key.
    pub key_url:     String,
    /// Whether a key is currently stored in the keychain.
    pub has_key:     bool,
}

/// Return metadata for all known API services, including whether a key is
/// stored for each.  Used by the Settings panel to populate the list.
#[tauri::command]
pub fn list_api_services() -> CmdResult<Vec<ApiServiceInfo>> {
    let services = [
        (
            keychain::VIRUSTOTAL,
            "VirusTotal",
            "Malware & URL reputation analysis (4 requests/min on free tier)",
            "https://www.virustotal.com/gui/user/settings/apikey",
        ),
    ];

    services
        .iter()
        .map(|(id, name, desc, url)| {
            let has_key = keychain::has_key(id)
                .map_err(|e| format!("Keychain error for '{id}': {e}"))?;
            Ok(ApiServiceInfo {
                id:          id.to_string(),
                name:        name.to_string(),
                description: desc.to_string(),
                key_url:     url.to_string(),
                has_key,
            })
        })
        .collect()
}

/// Check whether a key is stored for `service` without revealing its value.
#[tauri::command]
pub fn check_api_key(service: String) -> CmdResult<bool> {
    keychain::has_key(&service)
}

/// Retrieve the stored API key for `service`.
///
/// Returns `Some(key)` if present, `None` if the user has not set one.
#[tauri::command]
pub fn get_api_key(service: String) -> CmdResult<Option<String>> {
    keychain::get_key(&service).map_err(|e| e)
}

/// Store `key` for `service` in the platform keychain.
/// Overwrites any existing key.
#[tauri::command]
pub fn set_api_key(service: String, key: String) -> CmdResult<()> {
    if key.trim().is_empty() {
        return Err("API key must not be empty".into());
    }
    keychain::set_key(&service, key.trim()).map_err(|e| e)
}

/// Delete the stored API key for `service`.
/// Deleting a non-existent key is a no-op.
#[tauri::command]
pub fn delete_api_key(service: String) -> CmdResult<()> {
    keychain::delete_key(&service).map_err(|e| e)
}

// ── Pipeline Commands ─────────────────────────────────────────────────────────

use crate::pipeline::{Pipeline, PipelineRegistry, PipelineExecutor};

/// Get list of available pipelines
#[tauri::command]
pub fn list_pipelines(state: State<AppState>) -> Vec<Pipeline> {
    let registry = state.pipeline.read().map_err(|_| "Pipeline registry locked".to_string()).unwrap();
    registry.list().to_vec()
}

/// Get pipeline by ID
#[tauri::command]
pub fn get_pipeline(id: String, state: State<AppState>) -> CmdResult<Option<Pipeline>> {
    let registry = state.pipeline.read().map_err(|_| "Pipeline registry locked".to_string())?;
    Ok(registry.get(&id).cloned())
}

/// Create a new pipeline
#[tauri::command]
pub fn create_pipeline(
    name: String,
    description: String,
    steps: Vec<crate::pipeline::PipelineStep>,
    state: State<AppState>,
) -> CmdResult<Pipeline> {
    let mut pipeline = Pipeline::new(&name, &description);
    for step in steps {
        pipeline.add_step(step);
    }

    // Validate
    let errors = pipeline.validate();
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }

    let mut registry = state.pipeline.write().map_err(|_| "Pipeline registry locked".to_string())?;
    registry.add(pipeline.clone());
    Ok(pipeline)
}

/// Update an existing pipeline
#[tauri::command]
pub fn update_pipeline(
    id: String,
    name: Option<String>,
    description: Option<String>,
    steps: Option<Vec<crate::pipeline::PipelineStep>>,
    enabled: Option<bool>,
    state: State<AppState>,
) -> CmdResult<bool> {
    let mut registry = state.pipeline.write().map_err(|_| "Pipeline registry locked".to_string())?;
    
    let pipeline = registry.get(&id);
    if pipeline.is_none() {
        return Ok(false);
    }

    let mut pipeline = pipeline.unwrap().clone();
    if let Some(name) = name {
        pipeline.name = name;
    }
    if let Some(description) = description {
        pipeline.description = description;
    }
    if let Some(steps) = steps {
        pipeline.steps = steps;
    }
    if let Some(enabled) = enabled {
        pipeline.enabled = enabled;
    }
    pipeline.updated_at = chrono::Utc::now().to_rfc3339();

    // Validate
    let errors = pipeline.validate();
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }

    // Remove old and add updated
    registry.remove(&id);
    registry.add(pipeline);
    Ok(true)
}

/// Delete a pipeline
#[tauri::command]
pub fn delete_pipeline(id: String, state: State<AppState>) -> CmdResult<bool> {
    let mut registry = state.pipeline.write().map_err(|_| "Pipeline registry locked".to_string())?;
    Ok(registry.remove(&id))
}

/// Execute a pipeline on a seed node
#[tauri::command]
pub fn run_pipeline(
    pipeline_id: String,
    node_id: String,
    state: State<AppState>,
) -> CmdResult<crate::pipeline::PipelineExecution> {
    // Get pipeline
    let registry = state.pipeline.read().map_err(|_| "Pipeline registry locked".to_string())?;
    let pipeline = registry.get(&pipeline_id).cloned();
    drop(registry);

    let pipeline = pipeline.ok_or_else(|| format!("Pipeline '{}' not found", pipeline_id))?;

    // Get seed node
    let graph = state.graph.read().map_err(|_| lock_err())?;
    let seed_node = graph.find_node(&node_id).cloned();
    drop(graph);

    let seed_node = seed_node.ok_or_else(|| format!("Node '{}' not found", node_id))?;

    // Get transforms
    let transforms = state.registry.as_refs();
    
    // Execute pipeline
    let mut executor = PipelineExecutor::new(&pipeline, &seed_node);
    let result = executor.execute(&transforms);

    // Increment execution count
    {
        let mut registry = state.pipeline.write().map_err(|_| "Pipeline registry locked".to_string())?;
        if let Some(p) = registry.get(&pipeline_id) {
            let mut p = p.clone();
            p.execution_count += 1;
            registry.remove(&pipeline_id);
            registry.add(p);
        }
    }

    match result {
        Ok((nodes, edges)) => {
            // Add results to graph
            let mut graph = state.graph.write().map_err(|_| lock_err())?;
            for node in nodes {
                graph.add_node(node);
            }
            for edge in edges {
                graph.add_edge(edge);
            }
        }
        Err(_) => {} // Error already recorded in execution
    }

    Ok(executor.get_execution().clone())
}

/// Get built-in pipeline templates
#[tauri::command]
pub fn list_pipeline_templates() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "domain-enrichment",
            "name": "Domain Enrichment",
            "description": "Comprehensive domain intelligence gathering",
            "steps": [
                {"transform_name": "DNS Lookup", "input_type": "Domain", "description": "Resolve to IP addresses", "timeout": 30, "continue_on_error": false},
                {"transform_name": "GeoIP Lookup", "input_type": "IP", "description": "Geolocate IP addresses", "timeout": 30, "continue_on_error": true},
                {"transform_name": "WHOIS Lookup", "input_type": "Domain", "description": "Get domain registration info", "timeout": 30, "continue_on_error": true}
            ],
            "tags": ["dns", "domain", "enrichment"]
        }),
        serde_json::json!({
            "id": "ip-reconnaissance",
            "name": "IP Reconnaissance",
            "description": "Gather intelligence on IP addresses",
            "steps": [
                {"transform_name": "GeoIP Lookup", "input_type": "IP", "description": "Geolocate IP", "timeout": 30, "continue_on_error": false},
                {"transform_name": "Reverse DNS", "input_type": "IP", "description": "Find hostnames", "timeout": 30, "continue_on_error": true}
            ],
            "tags": ["ip", "recon", "security"]
        })
    ]
}


