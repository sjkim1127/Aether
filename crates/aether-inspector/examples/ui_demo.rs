use aether_core::{InjectionEngine, Template, InjectionContext, AetherConfig};
use aether_ai::OpenAiProvider;
use aether_inspector::{Inspector, InspectorServer};
use std::sync::Arc;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // Initialize tracing to see what's happening
    tracing_subscriber::fmt::init();

    // 1. Initialize Inspector (it internally uses Arc<DashMap>)
    let inspector = Inspector::new();
    
    // We need an Arc of inspector for the server
    let server_inspector = Arc::new(inspector.clone());

    // 2. Start Inspector Server in background
    tokio::spawn(async move {
        let server = InspectorServer::new(server_inspector);
        if let Err(e) = server.start(3000).await {
            eprintln!("Inspector server error: {}", e);
        }
    });

    // 3. Setup Engine with Inspector as Observer
    let provider = OpenAiProvider::from_env()?;
    let config = AetherConfig::default()
        .with_healing(true)
        .with_toon(true);
    
    // passing inspector directly since it implements EngineObserver and InjectionEngine will wrap it in Arc
    let engine = InjectionEngine::with_config(provider, config)
        .with_observer(inspector);

    println!("🚀 Aether Inspector Lab started at http://localhost:3000");
    println!("Press Ctrl+C to stop.");

    // 4. Run some generations to see them in UI
    let template = Template::new("
        import React from 'react';
        
        // {{AI:component:Generate a complex React component for an AI analytics dashboard with charts}}
        export const Dashboard = () => {
            return (
                <div className='p-8'>
                    {{AI:content:Generate the JSX content for the dashboard with dummy data}}
                </div>
            );
        };
    ").with_name("AI_Dashboard_Template");

    let context = InjectionContext::new()
        .with_framework("React")
        .with_language("TypeScript")
        .with_architecture("Tailwind CSS");

    println!("Generating code... check the browser!");
    let _Instance = engine.render_with_context(&template, context).await?;
    
    println!("Generation 1 complete.");

    // Wait a bit and do another one
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let template2 = Template::new("
        // {{AI:logic:Create a Rust function that implements a Red-Black tree}}
    ").with_name("Algorithms_Lib");

    let _Instance2 = engine.render(&template2).await?;
    println!("Generation 2 complete.");

    // Keep alive
    tokio::signal::ctrl_c().await?;
    Ok(())
}
