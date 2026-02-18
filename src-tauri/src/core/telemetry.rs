use chrono::Local;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize)]
pub struct TelemetryEvent {
    pub timestamp: String,
    pub event_type: String,
    pub details: String,
}

pub struct TelemetryManager {
    log_path: PathBuf,
}

impl TelemetryManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or(PathBuf::from("."));
        let log_path = app_dir.join("telemetry.jsonl");
        Self { log_path }
    }

    pub fn log_event(&self, event_type: &str, details: &str) {
        let event = TelemetryEvent {
            timestamp: Local::now().to_rfc3339(),
            event_type: event_type.to_string(),
            details: details.to_string(),
        };

        if let Ok(json) = serde_json::to_string(&event) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_path)
            {
                let _ = writeln!(file, "{}", json);
            }
        }
    }
}
