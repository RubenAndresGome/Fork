
mod core;
mod db;
use crate::core::orchestrator::Orchestrator;
use crate::db::Database;
use crate::core::local_llm::LocalInferenceEngine;
use std::sync::{Arc, Mutex};
use tauri::{State, Manager};

struct AppState {
    orchestrator: Mutex<Orchestrator>,
    db: Arc<Database>, // Database is thread-safe thanks to SqlitePool
    local_llm: Mutex<Option<LocalInferenceEngine>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_stats(state: State<'_, AppState>) -> Result<Vec<(String, i64, f64)>, String> {
    state.db.get_daily_stats().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_account(state: State<'_, AppState>, service: &str, username: &str, password: &str) -> Result<(), String> {
    // 1. Guardar password en keyring
    crate::core::auth::AuthManager::save_credentials(service, username, password).map_err(|e| e.to_string())?;
    // 2. Guardar metadata en DB
    state.db.add_account(service, username).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<(i64, String, String)>, String> {
    state.db.get_accounts().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_account(state: State<'_, AppState>, service: &str, username: &str) -> Result<(), String> {
    crate::core::auth::AuthManager::delete_credentials(service, username).map_err(|e| e.to_string())?;
    state.db.remove_account(service, username).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn run_code(language: &str, code: &str) -> Result<String, String> {
    let sandbox = crate::core::sandbox::SandboxManager::new().map_err(|e| e.to_string())?;
    match language {
        "python" => sandbox.run_python_code(code).await,
        "node" => sandbox.run_node_code(code).await,
        _ => Err(format!("Unsupported language: {}", language)),
    }
}

// Agent Commands
#[tauri::command]
async fn get_agents(state: State<'_, AppState>) -> Result<Vec<(i64, String, String, String, String, bool)>, String> {
    state.db.get_agents().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_agent(state: State<'_, AppState>, name: &str, description: &str, system_prompt: &str, default_model: &str) -> Result<i64, String> {
    state.db.create_agent(name, description, system_prompt, default_model).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_agent(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_agent(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_prompt(state: State<'_, AppState>, prompt: &str, model: &str, agent_id: Option<i64>) -> Result<String, String> {
    let mut final_prompt = prompt.to_string();
    let mut system_prompt: Option<String> = None;

    if let Some(id) = agent_id {
        // Recuperar prompt del agente (ineficiente hacer query cada vez, pero seguro)
        let agents = state.db.get_agents().await.map_err(|e| e.to_string())?;
        if let Some(agent) = agents.iter().find(|a| a.0 == id) {
            system_prompt = Some(agent.3.clone());
        }
    }
    
    if model == "openai_api" {
        // Recuperar API Key
        let api_key = crate::core::auth::AuthManager::get_password("openai_api", "default")
            .map_err(|_| "API Key not found. Please add account for 'openai_api' with username 'default'".to_string())?;

        // Llamada a API
        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(crate::core::openai::Message { role: "system".to_string(), content: sys.clone() });
        }
        messages.push(crate::core::openai::Message { role: "user".to_string(), content: final_prompt });

        let (response, cost) = crate::core::openai::send_chat_completion(&api_key, messages).await?;
        
        // Registrar costo real
        let _ = state.db.increment_usage(model, cost).await;
        
        return Ok(response);
    }

    if model == "local_phi2" {
        let engine_guard = state.local_llm.lock().unwrap();
        if let Some(engine) = engine_guard.as_ref() {
            let response = engine.generate(&final_prompt).map_err(|e| e.to_string())?;
            return Ok(response);
        } else {
            return Err("Local model not loaded. Please download/load it first.".to_string());
        }
    }

    // Para web, pre-pend system prompt
    if let Some(sys) = system_prompt {
        final_prompt = format!("Instrucciones del Sistema:\n{}\n\nUsuario:\n{}", sys, prompt);
    }

    // Registrar uso (costo 0.0 para scraping)
    let _ = state.db.increment_usage(model, 0.0).await; 

    let orchestrator = state.orchestrator.lock().unwrap();
    
    let action = match model {
        "chatgpt" => "chat_chatgpt",
        _ => "chat_deepseek",
    };

    let payload = serde_json::json!({ "prompt": final_prompt });
    orchestrator.send_command(action, Some(payload));
    
    Ok(format!("Prompt sent to {}: {}", model, prompt))
}

#[tauri::command]
async fn load_local_model(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let model_path = app_dir.join("models").join("phi-2-int4.onnx");
    let tokenizer_path = app_dir.join("models").join("tokenizer.json");

    if !model_path.exists() || !tokenizer_path.exists() {
        return Err("Model files not found in app data directory".to_string());
    }

    let engine = LocalInferenceEngine::new(&model_path, &tokenizer_path).map_err(|e| e.to_string())?;
    
    let mut local_store = state.local_llm.lock().unwrap();
    *local_store = Some(engine);
    
    Ok("Model loaded successfully".to_string())
}

#[tauri::command]
async fn check_updates(app_handle: tauri::AppHandle) -> Result<String, String> {
    crate::core::updater::check_and_update_adapters(&app_handle).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();
            tauri::async_runtime::block_on(async move {
                let db = Database::new(handle.clone()).await.expect("Failed to init DB");
                let orchestrator = Orchestrator::new();
                
                // Copy connection adapter if not exists
                // Note: In production we use handle.path().resource_dir()
                // For this MVP we will try to ensure the adapter is in AppData
                if let Ok(app_dir) = handle.path().app_data_dir() {
                     let adapters_dir = app_dir.join("adapters");
                     let _ = std::fs::create_dir_all(&adapters_dir);
                     let target_path = adapters_dir.join("playwright-service.js");
                     
                     if !target_path.exists() {
                         // Try to find it in resources
                         if let Ok(resource_path) = handle.path().resolve("playwright-service.js", tauri::path::BaseDirectory::Resource) {
                              let _ = std::fs::copy(resource_path, &target_path);
                         }
                     }
                }

                orchestrator.send_command("init", None);
                
                handle.manage(AppState {
                    orchestrator: Mutex::new(orchestrator),
                    db: Arc::new(db),
                    local_llm: Mutex::new(None),
                });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, send_prompt, get_stats, add_account, get_accounts, delete_account, run_code, get_agents, create_agent, delete_agent, load_local_model, check_updates])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
