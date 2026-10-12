pub mod commands;
pub mod entities;
pub mod graph;
pub mod keychain;
pub mod plugin_api;
pub mod provenance;
pub mod state;
pub mod tests;
pub mod transforms;

use commands::{
    // graph CRUD
    add_edge,
    add_node,
    // remote transforms
    add_remote_transform,
    // API keys
    check_api_key,
    clear_graph,
    collapse_nodes,
    delete_api_key,
    // file I/O
    export_csv_dialog,
    export_graphml_dialog,
    get_api_key,
    get_graph,
    import_csv_dialog,
    list_api_services,
    // entities
    list_entities,
    // machines
    list_machines,
    list_remote_transforms,
    // transforms
    list_transforms,
    load_demo_graph,
    load_graph_dialog,
    // plugins
    load_plugins_dir,
    remove_edge,
    remove_node,
    remove_remote_transform,
    run_machine,
    run_transform,
    save_graph_dialog,
    save_graph_to_path,
    set_api_key,
    update_node,
};
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // ── Auto-load plugins from the user's plugin directory ─────────────
            {
                let state = app.state::<AppState>();
                let app_data = app.handle().path().app_data_dir();

                if let Ok(base) = app_data {
                    let plugin_dir = base.join("plugins");
                    if plugin_dir.exists() {
                        // Plugins are arbitrary native code executed
                        // in-process — auto-load requires an explicit
                        // opt-in marker file.
                        let marker = plugin_dir.join("ENABLE_PLUGINS");
                        if !marker.is_file() {
                            log::info!(
                                "[plugin] auto-load disabled — create '{}' to enable it",
                                marker.display()
                            );
                        } else {
                            let (loaded, results) = plugin_api::scan_plugin_dir(&plugin_dir);
                            let count = loaded.len();
                            if let Ok(mut reg) = state.plugins.write() {
                                reg.extend(loaded);
                            }
                            for r in &results {
                                if r.ok {
                                    log::info!("[plugin] startup: {}", r.message);
                                } else {
                                    log::warn!("[plugin] startup: {}", r.message);
                                }
                            }
                            if count > 0 {
                                log::info!(
                                    "[plugin] {count} plugin(s) loaded from '{}'",
                                    plugin_dir.display()
                                );
                            }
                        }
                    }

                    // ── Load remote transforms from JSON ───────────────────────
                    let rt_path = base.join("remote_transforms.json");
                    if rt_path.exists() {
                        if let Ok(json) = std::fs::read_to_string(&rt_path) {
                            if let Ok(remote) = serde_json::from_str::<
                                Vec<crate::transforms::RemoteTransform>,
                            >(&json)
                            {
                                let count = remote.len();
                                if let Ok(mut reg) = state.remote_transforms.write() {
                                    *reg = remote;
                                    log::info!("[remote] {count} remote transform(s) loaded");
                                }
                            }
                        }
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // entities
            list_entities,
            // graph CRUD
            get_graph,
            add_node,
            update_node,
            remove_node,
            add_edge,
            remove_edge,
            clear_graph,
            collapse_nodes,
            // file I/O
            save_graph_dialog,
            save_graph_to_path,
            load_demo_graph,
            load_graph_dialog,
            export_csv_dialog,
            export_graphml_dialog,
            import_csv_dialog,
            // transforms (built-in + plugins)
            list_transforms,
            run_transform,
            add_remote_transform,
            list_remote_transforms,
            remove_remote_transform,
            // machines
            list_machines,
            run_machine,
            // plugin management
            load_plugins_dir,
            // API key management
            list_api_services,
            check_api_key,
            get_api_key,
            set_api_key,
            delete_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
