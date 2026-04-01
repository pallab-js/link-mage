pub mod commands;
pub mod entities;
pub mod graph;
pub mod keychain;
pub mod pipeline;
pub mod plugin_api;
pub mod provenance;
pub mod state;
pub mod tests;
pub mod transforms;

use tauri::Manager;
use state::AppState;
use commands::{
    // entities
    list_entities,
    // graph CRUD
    add_edge, add_node, clear_graph, collapse_nodes, get_graph, remove_edge,
    remove_node, save_graph_dialog, save_graph_to_path, update_node,
    // file I/O
    export_csv_dialog, export_graphml_dialog, import_csv_dialog, load_graph_dialog,
    // transforms
    list_transforms, run_transform,
    // remote transforms
    add_remote_transform, list_remote_transforms, remove_remote_transform,
    // machines
    list_machines, run_machine,
    // pipelines
    create_pipeline, delete_pipeline, get_pipeline, list_pipeline_templates,
    list_pipelines, run_pipeline, update_pipeline,
    // plugins
    load_plugins_dir,
    // API keys
    check_api_key, delete_api_key, get_api_key, list_api_services, set_api_key,
};

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
                let state     = app.state::<AppState>();
                let app_data  = app.handle().path().app_data_dir();

                if let Ok(base) = app_data {
                    let plugin_dir = base.join("plugins");
                    if plugin_dir.exists() {
                        let (loaded, results) = plugin_api::scan_plugin_dir(&plugin_dir);
                        let count = loaded.len();
                        if let Ok(mut reg) = state.plugins.write() {
                            reg.extend(loaded);
                        }
                        for r in &results {
                            if r.ok { log::info!("[plugin] startup: {}", r.message); }
                            else    { log::warn!("[plugin] startup: {}", r.message); }
                        }
                        if count > 0 {
                            log::info!(
                                "[plugin] {count} plugin(s) loaded from '{}'",
                                plugin_dir.display()
                            );
                        }
                    }

                    // ── Load remote transforms from JSON ───────────────────────
                    let rt_path = base.join("remote_transforms.json");
                    if rt_path.exists() {
                        if let Ok(json) = std::fs::read_to_string(&rt_path) {
                            if let Ok(remote) = serde_json::from_str::<Vec<crate::transforms::RemoteTransform>>(&json) {
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
            // pipelines
            list_pipelines,
            get_pipeline,
            create_pipeline,
            update_pipeline,
            delete_pipeline,
            run_pipeline,
            list_pipeline_templates,
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
