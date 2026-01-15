use aether_core::prelude::*;
use aether_macros::aether_secure;
use aether_ai::OpenAiProvider;
use dotenv::dotenv;

// This function's prompt will be encrypted at compile-time 
// if AETHER_SHIELD_KEY is set during the build.
#[aether_secure(prompt = "Determine if the user is an authorized admin based on their ID and secret key. Return true or false as a Rhai boolean.")]
async fn check_admin_access(user_id: String, secret: String) -> bool { todo!() }

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    println!("🛡️ Aether Shield: Prompt Encryption Demo");
    println!("---------------------------------------");

    // Setting the key for runtime decryption
    // In a real scenario, this would be set as an environment variable or derived from hardware.
    if std::env::var("AETHER_SHIELD_KEY").is_err() {
        std::env::set_var("AETHER_SHIELD_KEY", "super-secret-key-123");
        println!("Note: AETHER_SHIELD_KEY was not set. Using temporary runtime key.");
    }

    let user_id = "admin_01".to_string();
    let secret = "aether_2026".to_string();

    println!("Executing check_admin_access('{}', '***')...", user_id);
    
    // The macro will:
    // 1. Decrypt the prompt (if it was encrypted at compile time)
    // 2. Send it to AI
    // 3. Execute the returned Rhai script
    let is_admin = check_admin_access(user_id, secret).await;
    
    println!("Access Granted: {}", is_admin);

    Ok(())
}
