use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use std::path::Path;
use tokenizers::Tokenizer;

pub struct LocalInferenceEngine {
    session: Session,
    tokenizer: Tokenizer,
}

impl LocalInferenceEngine {
    pub fn new<P: AsRef<Path>>(model_path: P, tokenizer_path: P) -> Result<Self, String> {
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| e.to_string())?;

        // Configurar sesión ORT con DirectML si es posible, sino CPU
        let session = Session::builder()
            .map_err(|e: ort::Error| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e: ort::Error| e.to_string())?
            .with_intra_threads(4)
            .map_err(|e: ort::Error| e.to_string())?
            .commit_from_file(model_path)
            .map_err(|e| e.to_string())?;

        Ok(LocalInferenceEngine { session, tokenizer })
    }

    pub fn generate(&self, prompt: &str) -> Result<String, String> {
        // Simple generation placeholder
        // Dependency mismatch between ort and ndarray prevents actual inference logic for now.
        // We keep the session loaded but return a mock response.

        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| e.to_string())?;
        let input_ids = encoding.get_ids();

        // Suppress unused warning
        let _ = &self.session;

        // Mock response
        Ok(format!("Local Model (Mock): Processed {} tokens. Model loaded successfully (Inference skipped due to dependency mismatch).", input_ids.len()))
    }
}
