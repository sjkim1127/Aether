use aether_core::prelude::*;
use aether_core::provider::MockProvider;
use std::time::Instant;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Aether Incremental Rendering Demo");
    println!("------------------------------------");

    // 1. Setup Provider and Engine
    // We use a MockProvider to track exactly what's being generated
    let provider = MockProvider::new()
        .with_response("header", "/* Header v1 */")
        .with_response("footer", "/* Footer v1 */")
        .with_response("header_v2", "/* Header v2 */");
    
    let engine = InjectionEngine::new(provider);

    // 2. Initial Template
    let mut template = Template::new("{{AI:header}}\nContent\n{{AI:footer}}")
        .with_slot("header", "Generate a header comment")
        .with_slot("footer", "Generate a footer comment");

    // 3. First Render (Full)
    println!("🚀 First Render (Full):");
    let mut session = RenderSession::new();
    let start = Instant::now();
    let result1 = engine.render_incremental(&template, &mut session).await?;
    println!("Result 1:\n{}\nRender time: {:?}\n", result1, start.elapsed());

    // 4. Modify 'header', 'footer' remains the same
    println!("🔄 Modifying 'header' prompt, 'footer' remains unchanged...");
    template = template.with_slot("header", "Generate a header_v2 comment");

    // 5. Second Render (Incremental)
    println!("🚀 Second Render (Incremental):");
    let start = Instant::now();
    let result2 = engine.render_incremental(&template, &mut session).await?;
    println!("Result 2:\n{}\nRender time: {:?}\n", result2, start.elapsed());

    println!("✨ Observation: Only 'header' was re-generated. 'footer' result was reused instantly from the session cache.");

    Ok(())
}
