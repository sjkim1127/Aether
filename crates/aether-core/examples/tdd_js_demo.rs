use aether_core::prelude::*;
use aether_core::validation::MultiValidator;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("🧪 Aether TDD Self-Healing Demo (JavaScript)");
    println!("------------------------------------------");

    // 1. Define a TDD Test Harness (JavaScript / Node.js)
    let js_harness = r#"
        {{CODE}}

        // Simple test runner
        function assert(condition, msg) {
            if (!condition) {
                console.error("Assertion Failed: " + msg);
                process.exit(1);
            }
        }

        try {
            assert(add(1, 2) === 3, "1 + 2 should be 3");
            assert(add(-1, 1) === 0, "-1 + 1 should be 0");
            assert(add(0, 0) === 0, "0 + 0 should be 0");
            console.log("All tests passed!");
        } catch (e) {
            console.error(e);
            process.exit(1);
        }
    "#;

    // 2. Configure a Slot with TDD constraints
    let add_slot = Slot::new("add", "Write a JavaScript function `add(a, b)` that returns the sum of a and b.")
        .with_kind(SlotKind::JavaScript)
        .with_constraints(
            SlotConstraints::new()
                .test_harness(js_harness)
                .test_command("node {{FILE}}") // Optional, but shows customization
        );

    // 3. Setup Engine (Using OpenAI for demo)
    let provider = aether_ai::OpenAiProvider::from_env()?;
    let engine = InjectionEngine::new(provider)
        .with_validator(MultiValidator::new())
        .max_retries(2);

    let template = Template::new("// JS Add\n{{AI:add}}")
        .configure_slot(add_slot);

    println!("Generating JS code with TDD validation...");
    
    let result = engine.render(&template).await?;

    println!("\n✅ Successfully rendered with TDD passing:");
    println!("{}", result);

    Ok(())
}
