use serde::{Serialize, Deserialize};
use sqlx::{SqlitePool, Row};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: i64,
    pub collection: String,
    pub filename: String,
    pub content: String,
}

pub struct RagManager {
    pool: SqlitePool,
}

impl RagManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn ingest(&self, collection: &str, filename: &str, content: &str) -> Result<(), String> {
        // Simple chunking by paragraphs (double newline)
        let chunks: Vec<&str> = content.split("\n\n").collect();
        
        for chunk in chunks {
            if chunk.trim().len() < 10 { continue; } // Skip very short chunks
            
            sqlx::query("INSERT INTO documents (collection, filename, content) VALUES (?, ?, ?)")
                .bind(collection)
                .bind(filename)
                .bind(chunk.trim())
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn search(&self, collection: &str, query: &str, limit: i64) -> Result<Vec<DocumentChunk>, String> {
        // Basic keyword search using LIKE
        // For production RAG, we would use embeddings or FTS5
        let pattern = format!("%{}%", query);
        
        // Split query into keywords for better matching? 
        // For now, keep it simple: exact phrase or generic like
        
        let rows = sqlx::query("SELECT id, collection, filename, content FROM documents WHERE collection = ? AND content LIKE ? LIMIT ?")
            .bind(collection)
            .bind(&pattern)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
            
        let docs = rows.iter().map(|row| DocumentChunk {
            id: row.get("id"),
            collection: row.get("collection"),
            filename: row.get("filename"),
            content: row.get("content"),
        }).collect();
        
        Ok(docs)
    }

    #[allow(dead_code)]
    pub async fn get_collections(&self) -> Result<Vec<String>, String> {
        let rows = sqlx::query("SELECT DISTINCT collection FROM documents")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let collections = rows.iter().map(|row| row.get("collection")).collect();
        Ok(collections)
    }

    pub async fn get_documents(&self, collection: &str) -> Result<Vec<(i64, String, String)>, String> {
        let rows = sqlx::query_as::<_, (i64, String, String)>("SELECT id, filename, created_at FROM documents WHERE collection = ? ORDER BY created_at DESC")
             .bind(collection)
             .fetch_all(&self.pool)
             .await
             .map_err(|e| e.to_string())?;
        Ok(rows)
    }
}
