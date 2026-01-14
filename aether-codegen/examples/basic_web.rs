//! Basic web page generation example.
//!
//! This example demonstrates how to use Aether to generate a complete web page
//! with AI-generated content.
//!
//! # Prerequisites
//!
//! Set the `OPENAI_API_KEY` environment variable before running:
//!
//! ```bash
//! export OPENAI_API_KEY=your-api-key
//! cargo run --example basic_web
//! ```

use aether_ai::{InjectionEngine, OpenAiProvider, Template};
use aether_core::InjectionContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt::init();

    println!("🚀 Aether Codegen - Web Page Generator\n");

    // Create the AI provider
    let provider = OpenAiProvider::from_env()?;
    println!("✅ OpenAI provider initialized");

    // Create the template with AI slots
    let template = Template::new(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AI Generated Page</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: 'Segoe UI', system-ui, sans-serif; line-height: 1.6; }
        {{AI:styles:css}}
    </style>
</head>
<body>
    <header>
        {{AI:header:html}}
    </header>
    <main>
        {{AI:hero:html}}
        {{AI:features:html}}
    </main>
    <footer>
        {{AI:footer:html}}
    </footer>
</body>
</html>"#,
    )
    .with_name("landing_page")
    .with_slot(
        "styles",
        "Generate modern CSS styles for a landing page with a gradient background, smooth animations, and responsive design. Include styles for header, hero section, features grid, and footer.",
    )
    .with_slot(
        "header",
        "Generate a navigation header with a logo placeholder and 4 navigation links (Home, Features, Pricing, Contact). Style with flexbox.",
    )
    .with_slot(
        "hero",
        "Generate a hero section with a catchy headline about AI-powered development, a subtitle, and two buttons (Get Started, Learn More).",
    )
    .with_slot(
        "features",
        "Generate a features section with 3 feature cards, each with an emoji icon, title, and description about AI code generation capabilities.",
    )
    .with_slot(
        "footer",
        "Generate a simple footer with copyright text and social media icon placeholders.",
    );

    println!("📝 Template created with {} slots", template.slots.len());

    // Set up context for better generation
    let context = InjectionContext::new()
        .with_project("aether-demo")
        .with_language("html")
        .with_framework("vanilla");

    // Create the engine
    let engine = InjectionEngine::new(provider)
        .with_context(context)
        .parallel(true) // Generate all slots in parallel
        .max_retries(2);

    println!("⚡ Generating page with AI...\n");

    // Render the template
    let result = engine.render(&template).await?;

    // Output the result
    println!("Generated HTML:\n");
    println!("{}", result);

    // Optionally save to file
    tokio::fs::write("generated_page.html", &result).await?;
    println!("\n✅ Page saved to generated_page.html");

    Ok(())
}
