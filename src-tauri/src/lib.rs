
mod core;
mod db;
use crate::core::orchestrator::Orchestrator;
use crate::db::Database;
use crate::core::rag::RagManager;
use crate::core::local_llm::LocalInferenceEngine;
use std::sync::{Arc, Mutex};
use tauri::{State, Manager};

struct AppState {
    orchestrator: Mutex<Orchestrator>,
    db: Arc<Database>,
    local_llm: Mutex<Option<LocalInferenceEngine>>,
    telemetry: Arc<crate::core::telemetry::TelemetryManager>,
    rag: Arc<RagManager>,
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
    crate::core::auth::AuthManager::save_credentials(service, username, password).map_err(|e| e.to_string())?;
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
async fn update_agent(state: State<'_, AppState>, id: i64, name: &str, description: &str, system_prompt: &str, default_model: &str) -> Result<(), String> {
    state.db.update_agent(id, name, description, system_prompt, default_model).await.map_err(|e| e.to_string())
}

// RAG Commands
#[tauri::command]
async fn ingest_document(state: State<'_, AppState>, collection: &str, filename: &str, content: &str) -> Result<(), String> {
    state.rag.ingest(collection, filename, content).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn rag_search(state: State<'_, AppState>, collection: &str, query: &str) -> Result<Vec<crate::core::rag::DocumentChunk>, String> {
    state.rag.search(collection, query, 5).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_documents(state: State<'_, AppState>, collection: &str) -> Result<Vec<(i64, String, String)>, String> {
    state.rag.get_documents(collection).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_prompt(state: State<'_, AppState>, prompt: &str, model: &str, agent_id: Option<i64>, use_search: bool, collection: Option<String>) -> Result<String, String> {
    let mut final_prompt = prompt.to_string();
    let mut system_prompt: Option<String> = None;
    let mut context_text = String::new();

    if use_search {
        let col = collection.unwrap_or_else(|| "default".to_string());
        if let Ok(results) = state.rag.search(&col, prompt, 3).await {
            if !results.is_empty() {
                context_text.push_str("\n\nContexto Recuperado (RAG):\n");
                for (i, doc) in results.iter().enumerate() {
                    context_text.push_str(&format!("[{}] (from {}): {}\n", i+1, doc.filename, doc.content));
                }
                context_text.push_str("\n\n");
            }
        }
    }

    if let Some(id) = agent_id {
        let agents = state.db.get_agents().await.map_err(|e| e.to_string())?;
        if let Some(agent) = agents.iter().find(|a| a.0 == id) {
            system_prompt = Some(agent.3.clone());
        }
    }
    
    // Intelligent Scheduler & Telemetry
    let target_model = if model == "auto" {
        let is_local_loaded = state.local_llm.lock().unwrap().is_some();
        if is_local_loaded && prompt.len() < 150 {
            "local_phi2"
        } else {
            "cloud_deepseek"
        }
    } else {
        model
    };

    state.telemetry.log_event("prompt_received", &format!("Model: {} -> {}, Length: {}", model, target_model, prompt.len()));

    // Append context to prompt if using API or Local, or prepending to system prompt if possible
    // For simplicity, we'll prepend to the user prompt for now, or system prompt.
    // Let's prepend to final_prompt for everyone so it's included.
    if !context_text.is_empty() {
         final_prompt = format!("{}{}", context_text, final_prompt);
    }

    if target_model == "openai_api" {
        let api_key = crate::core::auth::AuthManager::get_password("openai_api", "default")
            .map_err(|_| "API Key not found. Please add account for 'openai_api' with username 'default'".to_string())?;

        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(crate::core::openai::Message { role: "system".to_string(), content: sys.clone() });
        }
        messages.push(crate::core::openai::Message { role: "user".to_string(), content: final_prompt });

        let (response, cost) = crate::core::openai::send_chat_completion(&api_key, messages).await?;
        let _ = state.db.increment_usage(target_model, cost).await;
        
        return Ok(response);
    }

    if target_model == "local_phi2" {
        let engine_guard = state.local_llm.lock().unwrap();
        if let Some(engine) = engine_guard.as_ref() {
            let start = std::time::Instant::now();
            let response = engine.generate(&final_prompt).map_err(|e| e.to_string())?;
            let duration = start.elapsed();
            state.telemetry.log_event("local_inference", &format!("Duration: {:?}, Chars: {}", duration, response.len()));
            return Ok(response);
        } else {
            return Err("Local model not loaded. Please download/load it first.".to_string());
        }
    }

    if let Some(sys) = system_prompt {
        final_prompt = format!("Instrucciones del Sistema:\n{}\n\nUsuario:\n{}", sys, final_prompt);
    }
    // Note: context is already in final_prompt

    let _ = state.db.increment_usage(target_model, 0.0).await; 

    let orchestrator = state.orchestrator.lock().unwrap();
    
    let action = match target_model {
        "chatgpt" => "chat_chatgpt",
        "cloud_glm" => "chat_glm",
        "cloud_kimi" => "chat_kimi",
        "cloud_deepseek" => "chat_deepseek",
        _ => "chat_deepseek",
    };

    let payload = serde_json::json!({ "prompt": final_prompt });
    orchestrator.send_command(action, Some(payload));
    
    Ok(format!("Prompt sent to {}: {}", target_model, prompt))
}

#[tauri::command]
async fn get_telemetry_log(app_handle: tauri::AppHandle) -> Result<String, String> {
    let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let log_path = app_dir.join("telemetry.jsonl");
    if log_path.exists() {
        std::fs::read_to_string(log_path).map_err(|e| e.to_string())
    } else {
        Ok("".to_string())
    }
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

#[tauri::command]
async fn unload_local_model(state: State<'_, AppState>) -> Result<String, String> {
    let mut local_store = state.local_llm.lock().unwrap();
    if local_store.is_none() {
         return Ok("No model loaded".to_string());
    }
    *local_store = None; // This drops the engine and frees memory
    Ok("Model unloaded successfully".to_string())
}

#[tauri::command]
async fn create_conversation(state: State<'_, AppState>, title: &str) -> Result<i64, String> {
    state.db.create_conversation(title).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_message(state: State<'_, AppState>, conversation_id: i64, role: &str, content: &str) -> Result<(), String> {
    state.db.add_message(conversation_id, role, content).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_messages(state: State<'_, AppState>, conversation_id: i64) -> Result<Vec<(String, String)>, String> {
    state.db.get_messages(conversation_id).await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();
            tauri::async_runtime::block_on(async move {
                let db = Database::new(handle.clone()).await.expect("Failed to init DB");
                let rag = RagManager::new(db.get_pool());
                
                // Copy connection adapter if not exists
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

                // Resolve path using updater (fixes unused warning and standardizes path)
                let adapter_path = crate::core::updater::get_adapter_path(&handle, "playwright-service.js");
                let orchestrator = Orchestrator::new(adapter_path);

                orchestrator.send_command("init", None);
                let telemetry = crate::core::telemetry::TelemetryManager::new(&handle);

                handle.manage(AppState {
                    orchestrator: Mutex::new(orchestrator),
                    db: Arc::new(db),
                    local_llm: Mutex::new(None),
                    telemetry: Arc::new(telemetry),
                    rag: Arc::new(rag),
                });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet, 
            send_prompt, 
            get_stats, 
            add_account, 
            get_accounts, 
            delete_account, 
            run_code, 
            get_agents, 
            create_agent, 
            delete_agent, 
            update_agent, 
            ingest_document, 
            rag_search, 
            get_documents, 
            load_local_model, 
            unload_local_model, 
            check_updates, 
            get_telemetry_log,
            create_conversation,
            add_message,
            get_messages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
