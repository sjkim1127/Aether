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
    },
    
    /// Initialize a new Aether configuration (Coming Soon)
    Init,
}

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
        Commands::Generate { template, output, provider, model, set } => {
            info!("Reading template from {:?}", template);
            
            // 1. Load Template
            let tmpl = Template::from_file(template)
                .await
                .context("Failed to load template file")?;
            
            // 2. Apply prompt overrides
            let mut tmpl = tmpl;
            for override_str in set {
                if let Some((slot, prompt)) = override_str.split_once('=') {
                     tmpl = tmpl.with_slot(slot, prompt);
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
                    let engine = InjectionEngine::new(p);
                    run_generation(engine, tmpl, output).await?;
                }
                ProviderType::Anthropic => {
                    let p = if let Some(m) = model {
                        aether_ai::anthropic(m)?
                    } else {
                        aether_ai::AnthropicProvider::from_env()?
                    };
                    let engine = InjectionEngine::new(p);
                    run_generation(engine, tmpl, output).await?;
                }
                ProviderType::Gemini => {
                    let p = if let Some(m) = model {
                        aether_ai::gemini(m)?
                    } else {
                        aether_ai::GeminiProvider::from_env()?
                    };
                    let engine = InjectionEngine::new(p);
                    run_generation(engine, tmpl, output).await?;
                }
                ProviderType::Ollama => {
                    let model_name = model.as_deref().unwrap_or("codellama");
                    let p = aether_ai::ollama(model_name);
                    let engine = InjectionEngine::new(p);
                    run_generation(engine, tmpl, output).await?;
                }
                ProviderType::Grok => {
                    let model_name = model.as_deref().unwrap_or("grok-1");
                    let p = aether_ai::grok(model_name)?;
                    let engine = InjectionEngine::new(p);
                    run_generation(engine, tmpl, output).await?;
                }
            }
        }
        Commands::Init => {
            println!("Initializing Aether project... (Not implemented yet)");
        }
    }

    Ok(())
}

async fn run_generation<P>(engine: InjectionEngine<P>, tmpl: Template, output: &Option<PathBuf>) -> Result<()> 
where 
    P: aether_core::AiProvider + Send + Sync + 'static,
{
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
    Ok(())
}
