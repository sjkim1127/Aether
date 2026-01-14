use aether_ai::{OpenAiProvider, AnthropicProvider, OllamaProvider, GeminiProvider};
use aether_core::{InjectionEngine, Template, ProviderConfig};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use dotenvy::dotenv;
use log::{info, error, debug};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate code from a template
    Generate {
        /// Path to the template file
        #[arg(short, long)]
        template: PathBuf,

        /// Output file path (optional, prints to stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// AI Provider to use
        #[arg(long, value_enum, default_value_t = ProviderType::Openai)]
        provider: ProviderType,

        /// Model name (optional, uses provider default if not specified)
        #[arg(short, long)]
        model: Option<String>,
        
        /// Specific prompt override for a slot (format: slot_name=prompt)
        #[arg(long)]
        set: Vec<String>,

        /// Enable streaming output (if applicable)
        #[arg(long)]
        stream: bool,

        /// Specific temperature for a slot (format: slot_name=temperature)
        #[arg(long)]
        temp: Vec<String>,

        /// Enable self-healing (auto-validate and fix code)
        #[arg(long)]
        heal: bool,

        /// Enable semantic caching to reduce costs
        #[arg(long)]
        cache: bool,

        /// Use TOON format for context optimization
        #[arg(long)]
        toon: bool,
    },
    
    /// Initialize a new Aether configuration (Coming Soon)
    Init,
}

use futures::stream::StreamExt;
use aether_core::validation::RustValidator;
use aether_core::cache::SemanticCache;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProviderType {
    Openai,
    Anthropic,
    Gemini,
    Ollama,
    Grok,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenv().ok();
    
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { template, output, provider, model, set, stream, heal, cache, toon, temp } => {
            info!("Reading template from {:?}", template);
            
            // 1. Load Template
            let tmpl = Template::from_file(template)
                .await
                .context("Failed to load template file")?;
            
            // 2. Apply prompt overrides
            let mut tmpl = tmpl;
            for override_str in set {
                if let Some((slot_name, prompt)) = override_str.split_once('=') {
                     tmpl = tmpl.with_slot(slot_name, prompt);
                }
            }
            
            for temp_str in temp {
                if let Some((slot_name, temp_val)) = temp_str.split_once('=') {
                    if let Ok(t) = temp_val.parse::<f32>() {
                        if let Some(slot) = tmpl.slots.get(slot_name).cloned() {
                            tmpl = tmpl.configure_slot(slot.with_temperature(t));
                        }
                    }
                }
            }

            // 3. Initialize Provider & Run
            info!("Initializing AI provider: {:?}", provider);
            
            match provider {
                ProviderType::Openai => {
                    let p = if let Some(m) = model {
                        aether_ai::openai(m)?
                    } else {
                        aether_ai::OpenAiProvider::from_env()?
                    };
                    let mut engine = InjectionEngine::new(p);
                    if *heal { engine = engine.with_validator(RustValidator); }
                    if *toon { engine = engine.with_toon(true); }
                    if *cache { engine = engine.with_cache(SemanticCache::new()?); }
                    run_generation(engine, tmpl, output, *stream).await?;
                }
                ProviderType::Anthropic => {
                    let p = if let Some(m) = model {
                        aether_ai::anthropic(m)?
                    } else {
                        aether_ai::AnthropicProvider::from_env()?
                    };
                    let mut engine = InjectionEngine::new(p);
                    if *heal { engine = engine.with_validator(RustValidator); }
                    if *toon { engine = engine.with_toon(true); }
                    if *cache { engine = engine.with_cache(SemanticCache::new()?); }
                    run_generation(engine, tmpl, output, *stream).await?;
                }
                ProviderType::Gemini => {
                    let p = if let Some(m) = model {
                        aether_ai::gemini(m)?
                    } else {
                        aether_ai::GeminiProvider::from_env()?
                    };
                    let mut engine = InjectionEngine::new(p);
                    if *heal { engine = engine.with_validator(RustValidator); }
                    if *toon { engine = engine.with_toon(true); }
                    if *cache { engine = engine.with_cache(SemanticCache::new()?); }
                    run_generation(engine, tmpl, output, *stream).await?;
                }
                ProviderType::Ollama => {
                    let model_name = model.as_deref().unwrap_or("codellama");
                    let p = aether_ai::ollama(model_name);
                    let mut engine = InjectionEngine::new(p);
                    if *heal { engine = engine.with_validator(RustValidator); }
                    if *toon { engine = engine.with_toon(true); }
                    if *cache { engine = engine.with_cache(SemanticCache::new()?); }
                    run_generation(engine, tmpl, output, *stream).await?;
                }
                ProviderType::Grok => {
                    let model_name = model.as_deref().unwrap_or("grok-1");
                    let p = aether_ai::grok(model_name)?;
                    let mut engine = InjectionEngine::new(p);
                    if *heal { engine = engine.with_validator(RustValidator); }
                    if *toon { engine = engine.with_toon(true); }
                    if *cache { engine = engine.with_cache(SemanticCache::new()?); }
                    run_generation(engine, tmpl, output, *stream).await?;
                }
            }
        }
        Commands::Init => {
            println!("Initializing Aether project... (Not implemented yet)");
        }
    }

    Ok(())
}

async fn run_generation<P>(engine: InjectionEngine<P>, tmpl: Template, output: &Option<PathBuf>, stream: bool) -> Result<()> 
where 
    P: aether_core::AiProvider + Send + Sync + 'static,
{
    if stream && tmpl.slots.len() == 1 {
        let slot_name = tmpl.slots.keys().next().unwrap().clone();
        info!("Streaming code generation for slot: {}", slot_name);
        
        let mut stream = engine.generate_slot_stream(&tmpl, &slot_name)?;
        let mut full_code = String::new();
        
        use std::io::{Write, stdout};
        let mut handle = stdout().lock();

        while let Some(result) = stream.next().await {
            let chunk = result?;
            full_code.push_str(&chunk.delta);
            
            if output.is_none() {
                print!("{}", chunk.delta);
                handle.flush()?;
            }
        }
        
        if output.is_none() {
            println!(""); // New line at end
        }

        if let Some(out_path) = output {
            let injections = std::collections::HashMap::from([(slot_name, full_code)]);
            let result = tmpl.render(&injections)?;
            tokio::fs::write(out_path, &result)
                .await
                .context("Failed to write output file")?;
            info!("Success! Output written to {:?}", out_path);
        }
    } else {
        // Fallback to normal rendering if multiple slots or streaming disabled
        if stream && tmpl.slots.len() > 1 {
            info!("Streaming requested but multiple slots found. Falling back to normal rendering.");
        }

        // 4. Render
        info!("Generating code... (this may take a while)");
        let result = engine.render(&tmpl).await.context("Code generation failed")?;

        // 5. Output
        if let Some(out_path) = output {
            tokio::fs::write(out_path, &result)
                .await
                .context("Failed to write output file")?;
            info!("Success! Output written to {:?}", out_path);
        } else {
            println!("{}", result);
        }
    }
    Ok(())
}
