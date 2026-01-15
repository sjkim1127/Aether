use aether_core::prelude::*;
use rhai::Scope;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("📜 Aether Script Integration Demo");
    println!("---------------------------------");

    // 1. Setup Agentic Runtime
    let provider = aether_ai::OpenAiProvider::from_env()?;
    let runtime = AetherAgenticRuntime::new(provider);

    // 2. Define an Aether Script (.ae)
    // Notice the @ai directive which is NOT standard Rhai!
    // It will be pre-processed and executed as a first-class AI call.
    let aether_script = r#"
        let user_id = "user_123";
        let profile = "User is a senior developer who loves Rust and agentic AI.";
        
        print("Analyzing profile for: " + user_id);
        
        // Use the @ai directive to call AI directly from the script!
        let summary = @ai("Summarize this profile in 10 words: " + profile);
        
        print("AI Summary: " + summary);
        
        let is_target = @ai("Is this person interested in Aether? Return 'yes' or 'no'. Context: " + summary);
        
        if is_target == "yes" {
            return "Promote Aether Enterprise";
        } else {
            return "Standard Onboarding";
        }
    "#;

    // 3. Execute Script
    println!("Executing Aether Script...");
    let mut scope = Scope::new();
    let result = runtime.execute(aether_script, &mut scope)?;

    println!("\n✅ Script Result: {}", result);

    Ok(())
}
