use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::fs;
use tauri::Manager;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new<R: tauri::Runtime>(app_handle: tauri::AppHandle<R>) -> Result<Self, Box<dyn std::error::Error>> {
        let app_dir = app_handle.path().app_data_dir()?;
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)?;
        }
        
        let db_path = app_dir.join("codechat.db");
        let db_url = format!("sqlite://{}", db_path.to_string_lossy());

        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            Sqlite::create_database(&db_url).await?;
        }

        let pool = SqlitePool::connect(&db_url).await?;
        
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(conversation_id) REFERENCES conversations(id)
            )"
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS usage_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                model_id TEXT NOT NULL,
                date DATE DEFAULT (DATE('now')),
                count INTEGER DEFAULT 1,
                estimated_cost REAL DEFAULT 0.0,
                UNIQUE(model_id, date)
            )"
        )
        .execute(&pool)
        .await?;

        // Migración simple para tablas existentes (si falla es que ya existe o tabla nueva)
        let _ = sqlx::query("ALTER TABLE usage_stats ADD COLUMN estimated_cost REAL DEFAULT 0.0")
            .execute(&pool)
            .await;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                service TEXT NOT NULL,
                username TEXT NOT NULL,
                is_active BOOLEAN DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(service, username)
            )"
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                system_prompt TEXT NOT NULL,
                default_model TEXT DEFAULT 'chatgpt',
                is_built_in BOOLEAN DEFAULT 0
            )"
        )
        .execute(&pool)
        .await?;

        // Insert Default Agents
        let _ = sqlx::query(
            "INSERT INTO agents (name, description, system_prompt, default_model, is_built_in) 
             VALUES ('Project Scaffolder', 'Genera estructuras de proyectos', 'Eres un experto en inicializar proyectos. Tu objetivo es generar comandos de terminal y estructuras de archivos para nuevos proyectos. Usa bloques de código para los comandos.', 'chatgpt', 1)
             ON CONFLICT DO NOTHING" // SQLite no tiene ON CONFLICT en INSERT simple sin constraint unique, pero ids autoincrement...
             // Mejor usamos WHERE NOT EXISTS o un SELECT count
        )
        .execute(&pool)
        .await;
        
        // Mejor aproximación para evitar duplicados si no hay unique constraint en name
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM agents WHERE name = 'Project Scaffolder'")
            .fetch_one(&pool)
            .await
            .unwrap_or((0,));
        
        if count.0 == 0 {
             let _ = sqlx::query(
                "INSERT INTO agents (name, description, system_prompt, default_model, is_built_in) 
                 VALUES ('Project Scaffolder', 'Genera estructuras de proyectos', 'Eres un experto en inicializar proyectos. Tu objetivo es generar comandos de terminal y estructuras de archivos para nuevos proyectos. Usa bloques de código para los comandos.', 'chatgpt', 1)"
            )
            .execute(&pool)
            .await;
        }

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                collection TEXT NOT NULL,
                filename TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&pool)
        .await?;

        Ok(Database { pool })
    }

    pub fn get_pool(&self) -> SqlitePool {
        self.pool.clone()
    }

    pub async fn create_conversation(&self, title: &str) -> Result<i64, sqlx::Error> {
        let id = sqlx::query("INSERT INTO conversations (title) VALUES (?)")
            .bind(title)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        Ok(id)
    }

    pub async fn add_message(&self, conversation_id: i64, role: &str, content: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO messages (conversation_id, role, content) VALUES (?, ?, ?)")
            .bind(conversation_id)
            .bind(role)
            .bind(content)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_messages(&self, conversation_id: i64) -> Result<Vec<(String, String)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, String)>("SELECT role, content FROM messages WHERE conversation_id = ? ORDER BY created_at ASC")
            .bind(conversation_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn increment_usage(&self, model_id: &str, cost: f64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO usage_stats (model_id, count, estimated_cost) VALUES (?, 1, ?)
             ON CONFLICT(model_id, date) DO UPDATE SET count = count + 1, estimated_cost = estimated_cost + ?"
        )
        .bind(model_id)
        .bind(cost)
        .bind(cost)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_daily_stats(&self) -> Result<Vec<(String, i64, f64)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, i64, f64)>("SELECT model_id, count, estimated_cost FROM usage_stats WHERE date = DATE('now')")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn add_account(&self, service: &str, username: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO accounts (service, username) VALUES (?, ?)")
            .bind(service)
            .bind(username)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_accounts(&self) -> Result<Vec<(i64, String, String)>, sqlx::Error> {
         let rows = sqlx::query_as::<_, (i64, String, String)>("SELECT id, service, username FROM accounts WHERE is_active = 1")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn remove_account(&self, service: &str, username: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM accounts WHERE service = ? AND username = ?")
            .bind(service)
            .bind(username)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Agents CRUD
    pub async fn create_agent(&self, name: &str, description: &str, system_prompt: &str, default_model: &str) -> Result<i64, sqlx::Error> {
        let id = sqlx::query("INSERT INTO agents (name, description, system_prompt, default_model) VALUES (?, ?, ?, ?)")
            .bind(name)
            .bind(description)
            .bind(system_prompt)
            .bind(default_model)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        Ok(id)
    }

    pub async fn get_agents(&self) -> Result<Vec<(i64, String, String, String, String, bool)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, String, String, String, String, bool)>("SELECT id, name, description, system_prompt, default_model, is_built_in FROM agents")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn delete_agent(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM agents WHERE id = ? AND is_built_in = 0")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_agent(&self, id: i64, name: &str, description: &str, system_prompt: &str, default_model: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE agents SET name = ?, description = ?, system_prompt = ?, default_model = ? WHERE id = ? AND is_built_in = 0")
            .bind(name)
            .bind(description)
            .bind(system_prompt)
            .bind(default_model)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
