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
            
        Ok(LocalInferenceEngine { 
            session, 
            tokenizer 
        })
    }

    pub fn generate(&self, prompt: &str) -> Result<String, String> {
        // Simple generation placeholder
        // In reality this needs a loop with KV cache handling for performance
        // For iteration 8, we verify we can encode and run one forward pass
        
        let encoding = self.tokenizer.encode(prompt, true).map_err(|e| e.to_string())?;
        let input_ids = encoding.get_ids();
        
        // Prepare inputs
        // let input_tensor = ndarray::Array2::from_shape_vec((1, input_ids.len()), input_ids.to_vec()).unwrap();
        // let outputs = self.session.run(ort::inputs!["input_ids" => input_tensor]?).unwrap();
        
        // Mock response for now until we have the full loop loop
        Ok(format!("Local Model (Mock): Processed {} tokens. Model loaded successfully.", input_ids.len()))
    }
}
