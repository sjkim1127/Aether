use aether_core::prelude::*;
use aether_core::validation::MultiValidator;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("🧪 Aether TDD Self-Healing Demo");
    println!("-------------------------------");

    // 1. Define a TDD Test Harness (Rust)
    // The {{CODE}} placeholder is where the AI code will be injected.
    let rust_harness = r#"
        {{CODE}}

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn test_fib() {
                assert_eq!(fib(0), 0);
                assert_eq!(fib(1), 1);
                assert_eq!(fib(5), 5);
                assert_eq!(fib(10), 55);
            }
        }
    "#;

    // 2. Configure a Slot with TDD constraints
    let fib_slot = Slot::new("fib", "Implement a recursive Fibonacci function named `fib` that takes an i32 and returns an i32.")
        .with_kind(SlotKind::Function)
        .with_constraints(
            SlotConstraints::new()
                .test_harness(rust_harness)
                // Default command for Rust TDD is used if not specified
        );

    // 3. Setup Engine (Using OpenAI for demo)
    let provider = aether_ai::OpenAiProvider::from_env()?;
    let engine = InjectionEngine::new(provider)
        .with_validator(MultiValidator::new())
        .max_retries(3);

    let template = Template::new("// Fibonacci implementation\n{{AI:fib}}")
        .configure_slot(fib_slot);

    println!("Generating code with TDD validation...");
    
    // The engine will:
    // 1. Generate code for 'fib'
    // 2. Inject it into the harness
    // 3. Run 'rustc --test'
    // 4. If it fails, send the error back to AI and retry
    let result = engine.render(&template).await?;

    println!("\n✅ Successfully rendered with TDD passing:");
    println!("{}", result);

    Ok(())
}
