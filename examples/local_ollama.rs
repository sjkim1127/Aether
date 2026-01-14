//! Local Ollama example.
//!
//! This example shows how to use Aether with a local Ollama instance
//! for offline code generation.
//!
//! # Prerequisites
//!
//! 1. Install Ollama: https://ollama.ai
//! 2. Pull a code model: `ollama pull codellama`
//! 3. Run this example: `cargo run --example local_ollama`

use aether_ai::{InjectionEngine, OllamaProvider, Template};
use aether_core::InjectionContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Aether Codegen - Local Ollama Example\n");

    // Create local Ollama provider
    let provider = OllamaProvider::new("codellama");

    // Check if Ollama is available
    use aether_core::AiProvider;
    if !provider.health_check().await.unwrap_or(false) {
        eprintln!("❌ Ollama is not running. Please start it first:");
        eprintln!("   ollama serve");
        eprintln!("   ollama pull codellama");
        return Ok(());
    }

    println!("✅ Ollama is running");

    // Set up context for Rust code generation
    let context = InjectionContext::new()
        .with_project("rust-utils")
        .with_language("rust");

    // Create engine with local provider
    let engine = InjectionEngine::new(provider)
        .with_context(context)
        .parallel(false) // Sequential for local models
        .max_retries(1);

    // Template for generating a utility function
    let template = Template::new(
        r#"/// {{AI:doc_comment}}
pub fn {{AI:function_name}}({{AI:parameters}}) -> {{AI:return_type}} {
    {{AI:implementation}}
}"#,
    )
    .with_slot("doc_comment", "Generate a doc comment for a function that validates email addresses")
    .with_slot("function_name", "Generate a function name like 'validate_email' or 'is_valid_email'")
    .with_slot("parameters", "Generate function parameters like 'email: &str'")
    .with_slot("return_type", "Generate return type like 'bool' or 'Result<bool, ValidationError>'")
    .with_slot("implementation", "Generate implementation that validates email format using regex");

    println!("⚡ Generating Rust code locally...\n");

    let result = engine.render(&template).await?;

    println!("Generated Rust Code:\n");
    println!("{}", result);

    Ok(())
}
