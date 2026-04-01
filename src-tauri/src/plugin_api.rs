//! Host-side dynamic plugin loading.
//!
//! # Plugin ABI contract
//!
//! Every plugin is a native shared library (`.dylib` on macOS, `.so` on Linux,
//! `.dll` on Windows) that exports **exactly seven** C-ABI symbols:
//!
//! | Symbol                    | C signature                                             | Notes                         |
//! |---------------------------|---------------------------------------------------------|-------------------------------|
//! | `plugin_api_version`      | `uint32_t (void)`                                       | Must equal [`API_VERSION`]    |
//! | `plugin_name`             | `const char* (void)`                                    | Static lifetime inside plugin |
//! | `plugin_description`      | `const char* (void)`                                    | Static lifetime inside plugin |
//! | `plugin_version`          | `const char* (void)`                                    | Static lifetime inside plugin |
//! | `plugin_applicable_types` | `const char* (void)`                                    | JSON array, static lifetime   |
//! | `plugin_run`              | `char* (const char* node_json)`                         | Heap-alloc; caller must free  |
//! | `plugin_free_string`      | `void (char*)`                                          | Free what `plugin_run` returns|
//!
//! `plugin_run` receives a UTF-8 JSON string (the seed node) and must return a
//! heap-allocated UTF-8 JSON string of the form:
//! ```json
//! { "new_nodes": [...], "new_edges": [...] }
//! ```
//! or, on failure:
//! ```json
//! { "error": "human-readable message" }
//! ```
//! The host will call `plugin_free_string` on the returned pointer once it has
//! copied the data.  The four `const char*` symbol functions return **static**
//! pointers that live for the lifetime of the loaded library — the host never
//! frees them.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};

use crate::graph::{Edge, Node};

// ── Version gate ──────────────────────────────────────────────────────────────

/// The host API version.  Plugins compiled against a different SDK version
/// are rejected at load time with a descriptive error message.
///
/// Increment this whenever the ABI contract changes in a breaking way.
pub const API_VERSION: u32 = 2;

// ── Raw C-ABI function types ──────────────────────────────────────────────────

type FnApiVersion = unsafe extern "C" fn() -> u32;
type FnCStr       = unsafe extern "C" fn() -> *const c_char;
type FnRun        = unsafe extern "C" fn(*const c_char) -> *mut c_char;
type FnFreeString = unsafe extern "C" fn(*mut c_char);

// ── Host-side metadata ────────────────────────────────────────────────────────

/// Owned, serializable plugin metadata — safe to send to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct PluginMeta {
    pub name:             String,
    pub description:      String,
    pub version:          String,
    pub applicable_types: Vec<String>,
    /// Absolute path of the `.dylib` / `.so` / `.dll` file.
    pub path:             PathBuf,
}

// ── Loaded plugin ─────────────────────────────────────────────────────────────

/// A dynamic plugin that has been loaded and validated.
///
/// Keeping this value alive keeps the shared library mapped into the process.
/// Dropping it unmaps the library — do not drop while a `run` call is in
/// progress.
pub struct LoadedPlugin {
    pub meta:   PluginMeta,
    // Kept alive to prevent the library from being unmapped.
    _lib:       Library,
    run_fn:     FnRun,
    free_fn:    FnFreeString,
}

// Raw pointers from `libloading` are not `Send + Sync` by default, but they
// are safe to share across threads as long as the Library lives — which it
// does for the lifetime of `AppState`.
unsafe impl Send for LoadedPlugin {}
unsafe impl Sync for LoadedPlugin {}

/// JSON shape that `plugin_run` must return.
#[derive(Deserialize)]
struct PluginResponse {
    #[serde(default)]
    new_nodes: Vec<Node>,
    #[serde(default)]
    new_edges: Vec<Edge>,
    error: Option<String>,
}

impl LoadedPlugin {
    /// Execute the plugin's transform.
    ///
    /// The node and graph are serialized to JSON and passed to the plugin.
    /// The plugin response is deserialized and returned as `(new_nodes, new_edges)`.
    pub fn run(
        &self,
        node:  &Node,
    ) -> Result<(Vec<Node>, Vec<Edge>), String> {
        let node_json  = serde_json::to_string(node)
            .map_err(|e| format!("serialize node: {e}"))?;

        let c_node  = CString::new(node_json) .map_err(|_| "node JSON has NUL")?;

        // SAFETY: we hold `_lib` alive, fn pointers are valid, inputs are
        // valid NUL-terminated UTF-8 strings.
        let raw = unsafe { (self.run_fn)(c_node.as_ptr()) };

        if raw.is_null() {
            return Err("plugin_run returned null".into());
        }

        // Copy the string, then hand the pointer back to the plugin for
        // deallocation.  Do this before any early return so we never leak.
        let json = unsafe { CStr::from_ptr(raw) }.to_string_lossy().into_owned();
        unsafe { (self.free_fn)(raw) };

        let resp: PluginResponse = serde_json::from_str(&json)
            .map_err(|e| format!("plugin returned malformed JSON: {e}\nraw: {json}"))?;

        if let Some(err) = resp.error {
            return Err(err);
        }

        Ok((resp.new_nodes, resp.new_edges))
    }
}

// ── Loading ───────────────────────────────────────────────────────────────────

/// Load and validate a single plugin from a shared library path.
///
/// Steps:
/// 1. `dlopen` the library.
/// 2. Resolve `plugin_api_version` and check it equals [`API_VERSION`].
/// 3. Read metadata via `plugin_name`, `plugin_description`, `plugin_version`,
///    `plugin_applicable_types`.
/// 4. Resolve and copy `plugin_run` and `plugin_free_string` function pointers.
///
/// # Safety
/// `libloading` is inherently unsafe.  Only load plugins from trusted sources.
pub fn load_plugin(path: &Path) -> Result<LoadedPlugin, String> {
    // SAFETY: loading arbitrary native code is always unsafe.
    let lib = unsafe { Library::new(path) }
        .map_err(|e| format!("dlopen '{}': {e}", path.display()))?;

    // ── 1. Version gate ───────────────────────────────────────────────────────
    let plugin_ver: u32 = unsafe {
        let f: Symbol<FnApiVersion> = lib
            .get(b"plugin_api_version\0")
            .map_err(|e| format!("missing 'plugin_api_version': {e}"))?;
        f()
    };

    if plugin_ver != API_VERSION {
        return Err(format!(
            "API version mismatch in '{}': host={API_VERSION}, plugin={plugin_ver}. \
             Recompile the plugin against link-mage-plugin-sdk v{API_VERSION}.",
            path.display()
        ));
    }

    // ── 2. Metadata ───────────────────────────────────────────────────────────
    let name        = call_static_cstr(&lib, b"plugin_name\0")?;
    let description = call_static_cstr(&lib, b"plugin_description\0")?;
    let version     = call_static_cstr(&lib, b"plugin_version\0")?;

    let types_json  = call_static_cstr(&lib, b"plugin_applicable_types\0")?;
    let applicable_types: Vec<String> = serde_json::from_str(&types_json).map_err(|e| {
        format!(
            "plugin_applicable_types in '{}' returned invalid JSON array: {e}",
            path.display()
        )
    })?;

    // ── 3. Run / free pointers ────────────────────────────────────────────────
    // We copy the raw function pointer out of the Symbol so we can store it
    // alongside the Library without lifetime gymnastics.  The pointer remains
    // valid for as long as `_lib` is alive.
    let run_fn: FnRun = unsafe {
        let f: Symbol<FnRun> = lib
            .get(b"plugin_run\0")
            .map_err(|e| format!("missing 'plugin_run': {e}"))?;
        *f
    };

    let free_fn: FnFreeString = unsafe {
        let f: Symbol<FnFreeString> = lib
            .get(b"plugin_free_string\0")
            .map_err(|e| format!("missing 'plugin_free_string': {e}"))?;
        *f
    };

    Ok(LoadedPlugin {
        meta: PluginMeta { name, description, version, applicable_types, path: path.to_path_buf() },
        _lib: lib,
        run_fn,
        free_fn,
    })
}

/// Call a symbol that returns a static `*const c_char` and convert it to an
/// owned `String`.  The pointer is borrowed immediately — it must remain valid
/// for the lifetime of the library (i.e. the plugin must return a pointer into
/// static storage, not a heap allocation).
fn call_static_cstr(lib: &Library, sym: &[u8]) -> Result<String, String> {
    let name = std::str::from_utf8(&sym[..sym.len() - 1]).unwrap_or("?");
    unsafe {
        let f: Symbol<FnCStr> = lib
            .get(sym)
            .map_err(|e| format!("missing '{name}': {e}"))?;
        let ptr = f();
        if ptr.is_null() {
            return Err(format!("'{name}' returned null"));
        }
        Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
    }
}

// ── Directory scanner ─────────────────────────────────────────────────────────

/// Result of attempting to load a single file from a plugin directory.
#[derive(Debug, Serialize)]
pub struct PluginLoadResult {
    pub path:    PathBuf,
    pub ok:      bool,
    pub message: String,
}

/// Scan `dir` for shared libraries and attempt to load each as a plugin.
///
/// Files are matched by platform extension: `.dylib` (macOS), `.so` (Linux),
/// `.dll` (Windows).  Every file is attempted; failures are captured as
/// `PluginLoadResult` entries rather than aborting the whole scan.
pub fn scan_plugin_dir(
    dir: &Path,
) -> (Vec<LoadedPlugin>, Vec<PluginLoadResult>) {
    let ext = if cfg!(target_os = "macos")   { "dylib" }
              else if cfg!(target_os = "windows") { "dll"   }
              else                                { "so"    };

    let mut plugins = Vec::new();
    let mut results = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e)  => e,
        Err(e) => {
            log::warn!("[plugin] cannot read dir '{}': {e}", dir.display());
            return (plugins, results);
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map(|x| x == ext).unwrap_or(false) {
            match load_plugin(&path) {
                Ok(p) => {
                    log::info!("[plugin] loaded '{}' v{}", p.meta.name, p.meta.version);
                    results.push(PluginLoadResult {
                        path: path.clone(),
                        ok: true,
                        message: format!("Loaded '{}' v{}", p.meta.name, p.meta.version),
                    });
                    plugins.push(p);
                }
                Err(e) => {
                    log::warn!("[plugin] failed to load '{}': {e}", path.display());
                    results.push(PluginLoadResult { path, ok: false, message: e });
                }
            }
        }
    }

    (plugins, results)
}

// ── Plugin registry ───────────────────────────────────────────────────────────

/// Holds all successfully loaded external plugins.
///
/// The host wraps this in `Arc<RwLock<PluginRegistry>>` so commands can share
/// read access while the load command takes a write lock.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<LoadedPlugin>,
}

impl PluginRegistry {
    pub fn new() -> Self { Self::default() }

    /// Replace all currently loaded plugins with `new_plugins`.
    /// Existing `LoadedPlugin` values are dropped, unmapping their libraries.
    pub fn replace_all(&mut self, new_plugins: Vec<LoadedPlugin>) {
        self.plugins = new_plugins;
    }

    /// Append additional plugins without clearing existing ones.
    pub fn extend(&mut self, extra: Vec<LoadedPlugin>) {
        self.plugins.extend(extra);
    }

    /// Return plugins whose `applicable_types` includes `node_type`.
    pub fn for_node_type<'a>(&'a self, node_type: &str) -> Vec<&'a LoadedPlugin> {
        self.plugins
            .iter()
            .filter(|p| p.meta.applicable_types.iter().any(|t| t == node_type))
            .collect()
    }

    /// Find a plugin by name (case-insensitive).
    pub fn find(&self, name: &str) -> Option<&LoadedPlugin> {
        let lower = name.to_lowercase();
        self.plugins.iter().find(|p| p.meta.name.to_lowercase() == lower)
    }

    /// Iterate over all loaded plugins.
    pub fn all(&self) -> &[LoadedPlugin] {
        &self.plugins
    }
}
