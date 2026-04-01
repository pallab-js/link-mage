//! # Link Mage Plugin SDK
//!
//! This crate provides the types, trait, and macro you need to write an
//! external transform plugin for Link Mage.
//!
//! ## Quick start
//!
//! ```toml
//! # Cargo.toml
//! [package]
//! name    = "my-plugin"
//! version = "0.1.0"
//! edition = "2021"
//!
//! [lib]
//! crate-type = ["cdylib"]   # ← must be cdylib so the linker emits a .dylib/.so/.dll
//!
//! [dependencies]
//! link-mage-plugin-sdk = { path = "../sdk" }   # or from crates.io
//! ```
//!
//! ```rust
//! // src/lib.rs
//! use link_mage_plugin_sdk::{declare_plugin, Node, RunContext, RunOutput};
//!
//! struct HelloPlugin;
//!
//! impl link_mage_plugin_sdk::PluginTransform for HelloPlugin {
//!     fn name(&self)        -> &str { "Hello World" }
//!     fn description(&self) -> &str { "Adds a greeting node to any IP" }
//!     fn version(&self)     -> &str { "0.1.0" }
//!     fn applicable_types(&self) -> &[&str] { &["IP"] }
//!
//!     fn run(&self, ctx: &RunContext) -> Result<RunOutput, String> {
//!         let ip = ctx.node_value();
//!         let greeting = Node::new_with_value("Note", &format!("Hello from {ip}!"));
//!         let edge     = ctx.edge_to(&greeting, "greeted");
//!         Ok(RunOutput { new_nodes: vec![greeting], new_edges: vec![edge] })
//!     }
//! }
//!
//! declare_plugin!(HelloPlugin, HelloPlugin);
//! ```
//!
//! Build your plugin:
//!
//! ```bash
//! cargo build --release
//! # macOS: target/release/libmy_plugin.dylib
//! # Linux: target/release/libmy_plugin.so
//! ```
//!
//! Drop the compiled library into the Link Mage plugin directory:
//! ```
//! ~/Library/Application Support/dev.linkmage.desktop/plugins/
//! ```
//!
//! The next time Link Mage launches it will load your plugin automatically.
//! You can also trigger a hot-reload via the `load_plugins_dir` Tauri command.
//!
//! ## API version
//!
//! [`API_VERSION`] is embedded in every plugin.  The host checks that it
//! matches before loading the library.  Increment this when the ABI changes.

pub use serde::{Deserialize, Serialize};
pub use serde_json;
// Re-export so the macro can reference it via `$crate::` — not strictly needed
// now that we use std::sync::OnceLock, but kept for potential future use.

// ── Stable ABI version ────────────────────────────────────────────────────────

/// Must equal `crate::plugin_api::API_VERSION` in the host.
/// Plugins compiled against a different version are rejected at load time.
pub const API_VERSION: u32 = 2;

// ── Shared data types ─────────────────────────────────────────────────────────
//
// These mirror the host's graph types exactly.  They are serialised as JSON
// across the FFI boundary, so layout compatibility is irrelevant — only the
// JSON field names matter.

/// A graph node — mirrors `graph::Node` in the host.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Node {
    pub id:          String,
    #[serde(rename = "node_type")]
    pub node_type:   String,
    pub properties:  std::collections::HashMap<String, String>,
}

impl Node {
    /// Create a node with a freshly generated UUID.
    pub fn new(node_type: &str) -> Self {
        Self {
            id:         uuid(),
            node_type:  node_type.to_string(),
            properties: std::collections::HashMap::new(),
        }
    }

    /// Create a node and set the `"value"` and `"label"` properties at once.
    pub fn new_with_value(node_type: &str, value: &str) -> Self {
        let mut n = Self::new(node_type);
        n.properties.insert("value".into(), value.into());
        n.properties.insert("label".into(), value.into());
        n
    }

    /// Convenience: read the `"value"` property (falls back to `id`).
    pub fn value(&self) -> &str {
        self.properties.get("value").map(|s| s.as_str()).unwrap_or(&self.id)
    }
}

/// A directed graph edge — mirrors `graph::Edge` in the host.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Edge {
    pub id:         String,
    pub source:     String,
    pub target:     String,
    pub label:      String,
    pub properties: std::collections::HashMap<String, String>,
}

impl Edge {
    /// Create an edge with a freshly generated UUID.
    pub fn new(source: &str, target: &str, label: &str) -> Self {
        Self {
            id:         uuid(),
            source:     source.to_string(),
            target:     target.to_string(),
            label:      label.to_string(),
            properties: std::collections::HashMap::new(),
        }
    }
}

// ── Run context ───────────────────────────────────────────────────────────────

/// Everything a transform receives when it is invoked on a node.
pub struct RunContext {
    /// The node being transformed.
    pub node:  Node,
}

impl RunContext {
    /// Shorthand for `self.node.value()`.
    pub fn node_value(&self) -> &str {
        self.node.value()
    }

    /// Build an edge from the source node to `target` with `label`.
    /// Uses `self.node.id` as the source automatically.
    pub fn edge_to(&self, target: &Node, label: &str) -> Edge {
        Edge::new(&self.node.id, &target.id, label)
    }

    /// Build an edge from `source` to the context node with `label`.
    pub fn edge_from(&self, source: &Node, label: &str) -> Edge {
        Edge::new(&source.id, &self.node.id, label)
    }
}

// ── Output ────────────────────────────────────────────────────────────────────

/// What a successful transform returns.
///
/// Both `new_nodes` and `new_edges` may be empty (e.g. if the transform only
/// enriches the existing node's properties — though currently there is no
/// mechanism to update existing nodes from a plugin; add enrichment nodes instead).
#[derive(Debug, Default, Serialize)]
pub struct RunOutput {
    pub new_nodes: Vec<Node>,
    pub new_edges: Vec<Edge>,
}

// ── Plugin trait ──────────────────────────────────────────────────────────────

/// Implement this trait and pass your type to [`declare_plugin!`].
///
/// All methods must be cheap and infallible (they are called once at load time
/// to build the static metadata).  Only `run` is called on every transform
/// invocation.
pub trait PluginTransform: Send + Sync + 'static {
    /// Human-readable display name shown in the context menu.
    fn name(&self) -> &str;

    /// One-sentence description shown as a tooltip in the UI.
    fn description(&self) -> &str;

    /// Semantic version of the plugin, e.g. `"0.1.0"`.
    fn version(&self) -> &str;

    /// Node types this transform applies to (e.g. `&["IP", "IPv6"]`).
    ///
    /// Return an empty slice to match all node types.
    fn applicable_types(&self) -> &[&str];

    /// Execute the transform.
    ///
    /// - On success return a [`RunOutput`] with any newly created nodes/edges.
    /// - On failure return an `Err(String)` with a human-readable message.
    ///
    /// **Do not panic** — panics across the FFI boundary are undefined
    /// behaviour.  Return `Err(...)` for all error conditions.
    fn run(&self, ctx: &RunContext) -> Result<RunOutput, String>;
}

// ── UUID helper ───────────────────────────────────────────────────────────────

/// Generate a v4 UUID string using the platform PRNG.
///
/// Intentionally minimal to avoid pulling in the full `uuid` crate.
pub fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Mix time + thread_id + some XOR to get enough entropy for our purpose.
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    let tid = {
        // Use a counter per thread as a poor-man's thread-local source.
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(0);
        CTR.fetch_add(1, Ordering::Relaxed)
    };
    let a = t.wrapping_mul(6364136223846793005).wrapping_add(tid);
    let b = a.wrapping_mul(0xdeadbeef_cafebabe).wrapping_add(0x1234_5678);
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (a >> 32) as u32,
        (a >> 16) as u16,
        (a & 0xfff) as u16,
        (0x8000u16 | ((b >> 48) as u16 & 0x3fff)),
        b & 0xffff_ffff_ffff,
    )
}

// ── declare_plugin! macro ─────────────────────────────────────────────────────

/// Register a [`PluginTransform`] implementation as a Link Mage plugin.
///
/// Place this macro call at the root of your `lib.rs` (not inside a module).
/// It expands to the seven `#[no_mangle] pub extern "C"` functions the host
/// expects.
///
/// ```rust
/// declare_plugin!(MyTransform, MyTransform { /* fields */ });
/// ```
///
/// - The first argument is the **type** of your transform struct.
/// - The second argument is an **expression** that constructs the singleton
///   instance (evaluated lazily on first use).
///
/// # Panics in metadata calls
/// The metadata functions (`plugin_name`, `plugin_description`, etc.) will
/// panic at startup if the returned strings contain interior NUL bytes (`\0`).
/// Do not put NUL bytes in your strings.
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:expr) => {
        // ── Singleton instance ────────────────────────────────────────────
        fn __plugin_instance() -> &'static $plugin_type {
            static INST: ::std::sync::OnceLock<$plugin_type> = ::std::sync::OnceLock::new();
            INST.get_or_init(|| $constructor)
        }

        // ── API version ───────────────────────────────────────────────────
        #[no_mangle]
        pub extern "C" fn plugin_api_version() -> u32 {
            $crate::API_VERSION
        }

        // ── Static string helpers ─────────────────────────────────────────
        //
        // Each function caches its CString in a OnceLock<CString> and returns
        // a raw pointer into it.  The pointer is valid for the lifetime of
        // the loaded library (i.e. the whole process lifetime for plugins).

        #[no_mangle]
        pub extern "C" fn plugin_name() -> *const ::std::os::raw::c_char {
            static CACHE: ::std::sync::OnceLock<::std::ffi::CString> = ::std::sync::OnceLock::new();
            CACHE.get_or_init(|| {
                ::std::ffi::CString::new($crate::PluginTransform::name(__plugin_instance()))
                    .expect("plugin name contains NUL byte")
            }).as_ptr()
        }

        #[no_mangle]
        pub extern "C" fn plugin_description() -> *const ::std::os::raw::c_char {
            static CACHE: ::std::sync::OnceLock<::std::ffi::CString> = ::std::sync::OnceLock::new();
            CACHE.get_or_init(|| {
                ::std::ffi::CString::new($crate::PluginTransform::description(__plugin_instance()))
                    .expect("plugin description contains NUL byte")
            }).as_ptr()
        }

        #[no_mangle]
        pub extern "C" fn plugin_version() -> *const ::std::os::raw::c_char {
            static CACHE: ::std::sync::OnceLock<::std::ffi::CString> = ::std::sync::OnceLock::new();
            CACHE.get_or_init(|| {
                ::std::ffi::CString::new($crate::PluginTransform::version(__plugin_instance()))
                    .expect("plugin version contains NUL byte")
            }).as_ptr()
        }

        /// Returns a JSON array of applicable node type strings, e.g. `["IP","IPv6"]`.
        #[no_mangle]
        pub extern "C" fn plugin_applicable_types() -> *const ::std::os::raw::c_char {
            static CACHE: ::std::sync::OnceLock<::std::ffi::CString> = ::std::sync::OnceLock::new();
            CACHE.get_or_init(|| {
                let types = $crate::PluginTransform::applicable_types(__plugin_instance());
                let json  = $crate::serde_json::to_string(types)
                    .expect("failed to serialize applicable_types");
                ::std::ffi::CString::new(json).expect("applicable_types JSON contains NUL byte")
            }).as_ptr()
        }

        // ── Transform execution ───────────────────────────────────────────
        //
        // `plugin_run` must not panic — panics across the FFI boundary are
        // undefined behaviour.  We wrap the call in catch_unwind.

        #[no_mangle]
        pub extern "C" fn plugin_run(
            node_json:  *const ::std::os::raw::c_char,
        ) -> *mut ::std::os::raw::c_char {
            use ::std::ffi::{CStr, CString};

            /// Encode a successful output or error as a JSON string and
            /// return it as a heap-allocated C string.
            fn encode(result: Result<$crate::RunOutput, String>) -> *mut ::std::os::raw::c_char {
                let json = match result {
                    Ok(out) => $crate::serde_json::to_string(&out)
                        .unwrap_or_else(|e| format!(r#"{{"error":"serialize error: {e}"}}"#)),
                    Err(msg) => format!(r#"{{"error":{}}}"#,
                        $crate::serde_json::to_string(&msg).unwrap_or_default()),
                };
                CString::new(json)
                    .unwrap_or_else(|_| {
                        CString::new(r#"{"error":"response contained NUL byte"}"#).unwrap()
                    })
                    .into_raw()
            }

            // Wrap in catch_unwind so that a panicking plugin doesn't abort
            // the host process.
            let result = ::std::panic::catch_unwind(|| {
                // SAFETY: host guarantees both pointers are valid NUL-terminated
                // UTF-8 strings for the duration of this call.
                let node_str  = unsafe { CStr::from_ptr(node_json)  }.to_str()
                    .map_err(|_| "node_json is not valid UTF-8".to_string())?;

                let node: $crate::Node = $crate::serde_json::from_str(node_str)
                    .map_err(|e| format!("deserialize node: {e}"))?;

                let ctx = $crate::RunContext { node };
                $crate::PluginTransform::run(__plugin_instance(), &ctx)
            });

            match result {
                Ok(inner)  => encode(inner),
                Err(_panic) => encode(Err("plugin panicked".to_string())),
            }
        }

        /// Free a string previously returned by `plugin_run`.
        ///
        /// The host **must** call this exactly once per string it receives
        /// from `plugin_run`.  Calling it with a null pointer is a no-op.
        #[no_mangle]
        pub extern "C" fn plugin_free_string(ptr: *mut ::std::os::raw::c_char) {
            if !ptr.is_null() {
                // SAFETY: ptr was created by CString::into_raw() in plugin_run.
                unsafe { drop(::std::ffi::CString::from_raw(ptr)) };
            }
        }
    };
}
