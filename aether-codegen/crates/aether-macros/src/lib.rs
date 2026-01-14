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
