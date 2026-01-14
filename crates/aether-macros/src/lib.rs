//! # Aether Macros
//!
//! Procedural macros for one-line AI code injection.
//!
//! This crate provides the `ai!` macro for compile-time code generation markers
//! and runtime injection helpers.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

/// Generate a template slot marker.
///
/// This macro creates a slot definition that will be filled by AI at runtime.
///
/// # Example
///
/// ```rust,ignore
/// use aether_macros::ai_slot;
///
/// // Creates a slot named "button" with the given prompt
/// let code = ai_slot!("button", "Create a submit button with hover effects");
/// ```
#[proc_macro]
pub fn ai_slot(input: TokenStream) -> TokenStream {
    let input_str = parse_macro_input!(input as LitStr);
    let prompt = input_str.value();

    let output = quote! {
        aether_core::Slot::new("generated", #prompt)
    };

    output.into()
}

/// Create an AI injection template inline.
///
/// # Example
///
/// ```rust,ignore
/// use aether_macros::ai_template;
///
/// let template = ai_template!("<div>{{AI:content}}</div>");
/// ```
#[proc_macro]
pub fn ai_template(input: TokenStream) -> TokenStream {
    let input_str = parse_macro_input!(input as LitStr);
    let content = input_str.value();

    let output = quote! {
        aether_core::Template::new(#content)
    };

    output.into()
}

/// One-line AI code generation (async).
///
/// This macro creates a future that generates code using the specified
/// provider and prompt.
///
/// # Example
///
/// ```rust,ignore
/// use aether_macros::ai;
/// use aether_ai::OpenAiProvider;
///
/// async fn example() {
///     let provider = OpenAiProvider::from_env().unwrap();
///     let code = ai!("Create a login form", provider).await.unwrap();
///     println!("{}", code);
/// }
/// ```
#[proc_macro]
pub fn ai(input: TokenStream) -> TokenStream {
    let input_tokens: proc_macro2::TokenStream = input.into();

    // Parse as: prompt, provider
    let output = quote! {
        {
            async {
                use aether_core::{InjectionEngine, Template};

                let (prompt, provider) = (#input_tokens);
                let template = Template::new("{{AI:generated}}")
                    .with_slot("generated", prompt);

                let engine = InjectionEngine::new(provider);
                engine.render(&template).await
            }
        }
    };

    output.into()
}

/// Mark a code section for AI generation (placeholder).
///
/// This attribute marks a function or item for AI-assisted generation.
/// Use with build tools that preprocess source files.
///
/// # Example
///
/// ```rust,ignore
/// #[ai_generate("Implement a function that validates email addresses")]
/// fn validate_email(email: &str) -> bool {
///     // AI will generate this implementation
///     todo!()
/// }
/// ```
#[proc_macro_attribute]
pub fn ai_generate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _prompt = parse_macro_input!(attr as LitStr);
    let item_tokens: proc_macro2::TokenStream = item.into();

    let output = quote! {
        // AI Generation prompt: #prompt
        #item_tokens
    };

    output.into()
}

/// Transform a function into a secure, polymorphic AI-powered runtime call.
/// 
/// This macro removes the function body and replaces it with logic that:
/// 1. Fetches a script from AI at runtime.
/// 2. Executes it using the AetherRuntime (Rhai).
/// 
/// # Example
/// 
/// ```rust,ignore
/// #[aether_secure(prompt = "Calculate complex score based on inputs", temp = 0.0)]
/// fn calculate_score(a: i64, b: i64) -> i64;
/// ```
#[proc_macro_attribute]
pub fn aether_secure(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_args = &input.sig.inputs;
    let fn_output = &input.sig.output;

    // Simplified attribute parsing (in production use syn::AttributeArgs)
    let attr_str = attr.to_string();
    let prompt = if let Some(p) = attr_str.split("prompt =").nth(1).and_then(|s| s.split('"').nth(1)) {
        p.to_string()
    } else {
        "Generate logic for this function".to_string()
    };

    let arg_names: Vec<_> = fn_args.iter().filter_map(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_id) = &*pat_type.pat {
                return Some(&pat_id.ident);
            }
        }
        None
    }).collect();

    let output = quote! {
        #fn_vis async fn #fn_name(#fn_args) #fn_output {
            use aether_core::prelude::*;
            use aether_core::AetherRuntime;
            use std::collections::HashMap;

            // 1. Setup Engine & Request Script (Dynamic Provider Selection)
            // We must handle rendering inside match arms because InjectionEngine<P> types differ.
            
            let provider_type = std::env::var("AETHER_PROVIDER").unwrap_or_else(|_| "openai".to_string());
            
            // Prepare template (common logic)
            let script_prompt = format!(
                "Implement this logic in Rhai script: {}. Output ONLY the raw Rhai script code. The inputs available are: {:?}. Return the result directly. Do not wrap in markdown.",
                #prompt,
                vec![#(stringify!(#arg_names)),*]
            );
            
            let template = Template::new("{{AI:script}}")
                .configure_slot(Slot::new("script", script_prompt).with_temperature(0.0));

            let script = match provider_type.to_lowercase().as_str() {
                "anthropic" | "claude" => {
                    let p = aether_ai::AnthropicProvider::from_env().expect("Anthropic Provider not configured");
                    let engine = InjectionEngine::new(p);
                    engine.render(&template).await.expect("AI script generation failed")
                },
                "gemini" => {
                    let p = aether_ai::GeminiProvider::from_env().expect("Gemini Provider not configured");
                    let engine = InjectionEngine::new(p);
                    engine.render(&template).await.expect("AI script generation failed")
                },
                "ollama" => {
                    let model = std::env::var("AETHER_MODEL").unwrap_or_else(|_| "llama3".to_string());
                    let p = aether_ai::OllamaProvider::new(&model);
                    let engine = InjectionEngine::new(p);
                    engine.render(&template).await.expect("AI script generation failed")
                },
                _ => {
                   let p = aether_ai::OpenAiProvider::from_env().expect("OpenAI Provider not configured");
                   let engine = InjectionEngine::new(p);
                   engine.render(&template).await.expect("AI script generation failed")
                }
            };

            // 3. Execute in Runtime
            let runtime = AetherRuntime::new();
            let mut inputs = HashMap::new();
            #(
                inputs.insert(stringify!(#arg_names).to_string(), rhai::Dynamic::from(#arg_names));
             )*

            let result = runtime.execute(&script, inputs).expect("Runtime execution failed");
            
            // 4. Return result (simplified cast, needs more robust handling for varied types)
            result.cast()
        }
    };

    output.into()
}
