use std::sync::Mutex;
use dashmap::DashMap;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use crate::Result;
use tracing::{debug, info};

/// Trait for prompt caching strategies.
pub trait Cache: Send + Sync {
    /// Try to retrieve a cached response for a prompt.
    fn get(&self, prompt: &str) -> Option<String>;
    
    /// Store a response in the cache.
    fn set(&self, prompt: &str, response: String);
}

/// A cache that uses semantic similarity to find matches.
/// Useful when prompts are slightly different but intent is the same.
pub struct SemanticCache {
    model: Mutex<TextEmbedding>,
    // Storage: Embedding -> Response
    // We use a simple in-memory map and search for now.
    storage: DashMap<String, (Vec<f32>, String)>,
    threshold: f32,
}

impl SemanticCache {
    /// Create a new semantic cache with default embedding model.
    pub fn new() -> Result<Self> {
        info!("Initializing semantic cache with local embedding model...");
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true)
        ).map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

        Ok(Self {
            model: Mutex::new(model),
            storage: DashMap::new(),
            threshold: 0.90, // Default 90% similarity
        })
    }

    /// Set similarity threshold (0.0 to 1.0).
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
        let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let norm_v1: f32 = v1.iter().map(|v| v * v).sum::<f32>().sqrt();
        let norm_v2: f32 = v2.iter().map(|v| v * v).sum::<f32>().sqrt();
        dot_product / (norm_v1 * norm_v2)
    }
}

impl Cache for SemanticCache {
    fn get(&self, prompt: &str) -> Option<String> {
        let mut model = self.model.lock().ok()?;
        let embedding = model.embed(vec![prompt], None).ok()?.first()?.clone();
        
        // Linear search for similarity (O(N) - fine for small/medium local caches)
        let mut best_match: Option<(f32, String)> = None;

        for entry in self.storage.iter() {
            let (stored_embedding, response) = entry.value();
            let similarity = Self::cosine_similarity(&embedding, stored_embedding);
            
            if similarity >= self.threshold {
                if best_match.as_ref().map_or(true, |(score, _)| similarity > *score) {
                    best_match = Some((similarity, response.clone()));
                }
            }
        }

        if let Some((score, response)) = best_match {
            debug!("Semantic cache hit! Similarity: {:.2}", score);
            Some(response)
        } else {
            None
        }
    }

    fn set(&self, prompt: &str, response: String) {
        let mut model = match self.model.lock() {
            Ok(m) => m,
            Err(_) => return,
        };
        if let Ok(embeddings) = model.embed(vec![prompt], None) {
            if let Some(embedding) = embeddings.first() {
                self.storage.insert(prompt.to_string(), (embedding.clone(), response));
            }
        }
    }
}

/// A simple exact match cache.
pub struct ExactCache {
    storage: DashMap<String, String>,
}

impl ExactCache {
    pub fn new() -> Self {
        Self { storage: DashMap::new() }
    }
}

impl Cache for ExactCache {
    fn get(&self, prompt: &str) -> Option<String> {
        self.storage.get(prompt).map(|v| v.value().clone())
    }

    fn set(&self, prompt: &str, response: String) {
        self.storage.insert(prompt.to_string(), response);
    }
}

/// A hybrid cache that balances speed (exact) and flexibility (semantic).
pub struct TieredCache {
    exact: ExactCache,
    semantic: SemanticCache,
}

impl TieredCache {
    /// Create a new tiered cache.
    pub fn new() -> Result<Self> {
        Ok(Self {
            exact: ExactCache::new(),
            semantic: SemanticCache::new()?,
        })
    }
}

impl Cache for TieredCache {
    fn get(&self, prompt: &str) -> Option<String> {
        // 1. Try exact match first (O(1), very fast)
        if let Some(res) = self.exact.get(prompt) {
            return Some(res);
        }

        // 2. Fallback to semantic similarity (O(N) + Embedding overhead)
        self.semantic.get(prompt)
    }

    fn set(&self, prompt: &str, response: String) {
        // Store in both for maximum hits
        self.exact.set(prompt, response.clone());
        self.semantic.set(prompt, response);
    }
}
