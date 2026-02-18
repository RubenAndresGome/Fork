use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Serialize, Deserialize)]
struct PlaywrightCommand {
    action: String,
    payload: Option<serde_json::Value>,
}

pub struct Orchestrator {
    stdin: Arc<Mutex<Option<std::process::ChildStdin>>>,
}

impl Orchestrator {
    pub fn new(adapter_path: PathBuf) -> Self {
        // Validation moved to lib.rs / updater.rs

        let mut child = Command::new("node")
            .arg(adapter_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start node service"); // Changed service to process

        let stdin = child.stdin.take().expect("Failed to open stdin"); // Take stdin handle / Tomar handle stdin
        let stdout = child.stdout.take().expect("Failed to open stdout");

        // Hilo para leer stdout del proceso Node y loguearlo o emitirlo
        // HINT/PISTA: We spawn a thread to read stdout asynchronously. This prevents blocking the main thread
        // while waiting for Node.js output. We use channels or direct calls for communication.
        // Lanzamos un hilo para leer stdout asíncronamente. Previene bloquear el hilo principal
        // mientras esperamos salida de Node.js.
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    println!("[Node]: {}", l);
                    // Aquí podríamos emitir eventos a la UI si tuviéramos acceso al handle de Tauri
                }
            }
        });

        Orchestrator {
            stdin: Arc::new(Mutex::new(Some(stdin))),
        }
    }

    pub fn send_command(&self, action: &str, payload: Option<serde_json::Value>) {
        // Security Check
        if action == "navigate" {
            if let Some(p) = &payload {
                if let Some(url) = p.get("url").and_then(|u| u.as_str()) {
                    if !crate::core::security::SecurityPolicy::is_url_allowed(url) {
                        eprintln!("Security Violation: Blocked navigation to {}", url);
                        return;
                    }
                }
            }
        }

        let cmd = PlaywrightCommand {
            action: action.to_string(),
            payload,
        };
        let json = serde_json::to_string(&cmd).unwrap();

        if let Ok(mut guard) = self.stdin.lock() {
            if let Some(stdin) = guard.as_mut() {
                writeln!(stdin, "{}", json).expect("Failed to write to node stdin");
            }
        }
    }
}
