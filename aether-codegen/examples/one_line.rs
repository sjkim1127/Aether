//! One-line code generation example.
//!
//! This example demonstrates the simplest possible usage of Aether
//! with just one line of code to generate content.

use aether_ai::{InjectionEngine, OpenAiProvider, Template};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Aether Codegen - One-Line Generation\n");

    // === Method 1: Using the inject! macro (when available) ===
    // let code = aether_core::inject!("Create a button", provider);

    // === Method 2: One-liner with template ===
    let provider = OpenAiProvider::from_env()?;
    
    // Generate code with just one line
    let button = InjectionEngine::new(provider.clone())
        .render(
            &Template::new("{{AI:code}}")
                .with_slot("code", "Create an HTML button with hover effect"),
        )
        .await?;

    println!("Generated Button:\n{}\n", button);

    // === Method 3: Using the convenience function ===
    let login_form = InjectionEngine::new(provider.clone())
        .render(
            &Template::new("{{AI:form}}")
                .with_slot("form", "Create a login form with email, password, and submit button"),
        )
        .await?;

    println!("Generated Login Form:\n{}\n", login_form);

    // === Method 4: Chained one-liner ===
    let nav = InjectionEngine::new(provider)
        .render(
            &Template::new("{{AI:nav:html}}")
                .with_slot("nav", "Create a responsive navigation bar with 5 links"),
        )
        .await?;

    println!("Generated Navigation:\n{}", nav);

    Ok(())
}
