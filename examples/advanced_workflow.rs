use aether_ai::AnthropicProvider;
use aether_core::{
    cache::SemanticCache,
    validation::RustValidator,
    InjectionEngine, Template, Slot,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Setup Provider (using Claude 4.5 for high-quality logic)
    let provider = AnthropicProvider::from_env_with_model("claude-4.5-sonnet-latest")?;

    // 2. Initialize Engine with Premium Features
    let engine = InjectionEngine::new(provider)
        .with_cache(SemanticCache::new()?) // Enable Semantic Cache (saves costs)
        .with_validator(RustValidator)    // Enable Self-Healing for Rust code
        .with_toon(true)                  // Enable TOON optimization
        .parallel(true);                  // Generate slots concurrently

    // 3. Define a Template with per-slot Temperature
    // We want a creative name but very strict logic.
    let template_str = r#"
/*
 * Project: {{AI:project_name}}
 * Description: AI-generated module with validated logic.
 */

pub fn calculate_logic(input: i32) -> i32 {
    {{AI:logic_body}}
}
"#;

    let mut template = Template::new(template_str);

    // Slot 1: Creative project name (High Temperature)
    template = template.configure_slot(
        Slot::new("project_name", "Generate a creative, futuristic name for a Rust math module")
            .with_temperature(0.9)
    );

    // Slot 2: Strict mathematical logic (Low Temperature + Self-Healing)
    template = template.configure_slot(
        Slot::new("logic_body", "Implement a complex mathematical recursion that returns input * 2 safely.")
            .with_temperature(0.0) // Zero entropy for crystalline logic
    );

    println!("🚀 Generating advanced module with Aether...");

    // 4. Render with full power
    let result = engine.render(&template).await?;

    println!("\n--- Generated Output ---\n");
    println!("{}", result);
    println!("\n------------------------\n");
    println!("✅ Done! Caching, Healing, and Per-Slot Temperature all worked together.");

    Ok(())
}
