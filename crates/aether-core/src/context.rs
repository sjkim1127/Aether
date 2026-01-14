//! Injection context for code generation.
//!
//! Provides context information to AI for better code generation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context for AI code injection.
///
/// This provides additional information to the AI model to help generate
/// more relevant and accurate code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InjectionContext {
    /// Project name or identifier.
    pub project: Option<String>,

    /// Target language (e.g., "rust", "html", "typescript").
    pub language: Option<String>,

    /// Framework being used (e.g., "react", "vue", "actix-web").
    pub framework: Option<String>,

    /// Coding style preferences.
    pub style: Option<StyleGuide>,

    /// Surrounding code context.
    pub surrounding_code: Option<String>,

    /// Import statements available.
    pub available_imports: Vec<String>,

    /// Custom variables for template expansion.
    pub variables: HashMap<String, String>,

    /// Additional metadata.
    pub extra: HashMap<String, serde_json::Value>,
}

/// Coding style preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleGuide {
    /// Indentation style (spaces or tabs).
    pub indent: IndentStyle,

    /// Maximum line length.
    pub max_line_length: Option<usize>,

    /// Whether to use semicolons (for JS/TS).
    pub semicolons: Option<bool>,

    /// Quote style for strings.
    pub quote_style: Option<QuoteStyle>,

    /// Naming convention.
    pub naming_convention: Option<NamingConvention>,
}

/// Indentation style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndentStyle {
    /// Use spaces with specified count.
    Spaces(u8),
    /// Use tabs.
    Tabs,
}

/// Quote style for strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuoteStyle {
    Single,
    Double,
}

/// Naming convention.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamingConvention {
    CamelCase,
    PascalCase,
    SnakeCase,
    KebabCase,
}

impl InjectionContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the project name.
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Set the target language.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set the framework.
    pub fn with_framework(mut self, framework: impl Into<String>) -> Self {
        self.framework = Some(framework.into());
        self
    }

    /// Set the style guide.
    pub fn with_style(mut self, style: StyleGuide) -> Self {
        self.style = Some(style);
        self
    }

    /// Add surrounding code context.
    pub fn with_surrounding_code(mut self, code: impl Into<String>) -> Self {
        self.surrounding_code = Some(code.into());
        self
    }

    /// Add an available import.
    pub fn add_import(mut self, import: impl Into<String>) -> Self {
        self.available_imports.push(import.into());
        self
    }

    /// Set a variable.
    pub fn set_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Convert context to a prompt string for AI.
    pub fn to_prompt(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref project) = self.project {
            parts.push(format!("Project: {}", project));
        }

        if let Some(ref lang) = self.language {
            parts.push(format!("Language: {}", lang));
        }

        if let Some(ref fw) = self.framework {
            parts.push(format!("Framework: {}", fw));
        }

        if let Some(ref style) = self.style {
            let mut style_parts = Vec::new();
            match &style.indent {
                IndentStyle::Spaces(n) => style_parts.push(format!("{} spaces indent", n)),
                IndentStyle::Tabs => style_parts.push("tabs indent".to_string()),
            }
            if let Some(max) = style.max_line_length {
                style_parts.push(format!("max {} chars per line", max));
            }
            if !style_parts.is_empty() {
                parts.push(format!("Style: {}", style_parts.join(", ")));
            }
        }

        if !self.available_imports.is_empty() {
            parts.push(format!("Available imports: {}", self.available_imports.join(", ")));
        }

        if let Some(ref code) = self.surrounding_code {
            parts.push(format!("Surrounding code:\n```\n{}\n```", code));
        }

        parts.join("\n")
    }
}

impl Default for StyleGuide {
    fn default() -> Self {
        Self {
            indent: IndentStyle::Spaces(4),
            max_line_length: Some(100),
            semicolons: None,
            quote_style: None,
            naming_convention: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let ctx = InjectionContext::new()
            .with_project("my-app")
            .with_language("typescript")
            .with_framework("react");

        assert_eq!(ctx.project, Some("my-app".to_string()));
        assert_eq!(ctx.language, Some("typescript".to_string()));
        assert_eq!(ctx.framework, Some("react".to_string()));
    }

    #[test]
    fn test_context_to_prompt() {
        let ctx = InjectionContext::new()
            .with_project("test")
            .with_language("rust");

        let prompt = ctx.to_prompt();
        assert!(prompt.contains("Project: test"));
        assert!(prompt.contains("Language: rust"));
    }
}
