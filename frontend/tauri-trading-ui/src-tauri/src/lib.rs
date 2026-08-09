//! FX eTrading Tauri shell — native desktop host for the trading UI.
//!
//! Security posture:
//! - Minimal capability set (`capabilities/default.json`)
//! - Strict CSP in `tauri.conf.json` (connect only to local gateway / loopback)
//! - Observability scrapes loopback `/health` + `/metrics` in-process (no Prometheus UI)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod observability;

use observability::ObsCollector;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let collector = ObsCollector::new().expect("observability collector");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(collector)
        .invoke_handler(tauri::generate_handler![observability::obs_collect])
        .run(tauri::generate_context!())
        .expect("error while running FX eTrading Tauri application");
}
