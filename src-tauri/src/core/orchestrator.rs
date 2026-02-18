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
    pub fn new() -> Self {
        // The stdin handle needs to be moved out of the thread::spawn to be accessible by Orchestrator
        // This part of the original code needs to be re-evaluated based on the new structure.
        // For now, I'll assume the stdin handle is still taken directly from the child process
        // in the main thread, or passed back via the channel (which isn't fully implemented here).
        // Given the user's provided snippet, the `stdin` variable is declared inside the new `thread::spawn`.
        // This means `Orchestrator { stdin: Arc::new(Mutex::new(stdin)), }` would not compile.
        // I will revert the `thread::spawn` around the `Command::new` for now, as the user's snippet
        // seems to imply the `Command::new` and `stdin` capture still happens in the main thread,
        // and only the `stdout` reading is in a separate thread.
        // The `let (tx, rx) = mpsc::channel();` and the outer `thread::spawn` seem to be
        // an incomplete thought or a misunderstanding of how `stdin` would be passed back.

        // Reverting to the original structure for `Command::new` and `stdin` capture,
        // but incorporating the new path resolution.

        let adapter_path = if let Some(mut path) = dirs::data_dir() {
            path.push("CodeChatUniversal");
            path.push("adapters");
            path.push("playwright-service.js");
            if path.exists() {
                path
            } else {
                PathBuf::from("../src-core/playwright-service.js")
            }
        } else {
            PathBuf::from("../src-core/playwright-service.js")
        };

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
            stdin: Arc::new(Mutex::new(stdin)),
        }
    }

    pub fn send_command(&self, action: &str, payload: Option<serde_json::Value>) {
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
