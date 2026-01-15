//! Template parsing and management.
//!
//! Templates contain slots marked with `{{AI:slot_name}}` syntax that will be
//! replaced with AI-generated code.

use crate::{AetherError, Result, Slot, SlotKind};
use regex::Regex;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Pattern for matching AI slots in templates.
/// Format: {{AI:slot_name}} or {{AI:slot_name:kind}}
const SLOT_PATTERN: &str = r"\{\{AI:([a-zA-Z_][a-zA-Z0-9_]*)(?::([a-zA-Z]+))?\}\}";

static SLOT_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_slot_regex() -> &'static Regex {
    SLOT_REGEX.get_or_init(|| Regex::new(SLOT_PATTERN).expect("Invalid slot pattern regex"))
}

/// Represents a template with AI injection slots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Original template content.
    pub content: String,

    /// Name of this template.
    pub name: String,

    /// Slots found in the template.
    pub slots: HashMap<String, Slot>,

    /// Template metadata.
    pub metadata: TemplateMetadata,
}

/// Metadata about a template.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template description.
    pub description: Option<String>,

    /// Template language (html, rust, js, etc.).
    pub language: Option<String>,

    /// Template author.
    pub author: Option<String>,

    /// Template version.
    pub version: Option<String>,
}

/// A parsed slot location in the template.
#[derive(Debug, Clone)]
pub struct SlotLocation {
    /// Slot name.
    pub name: String,

    /// Start position in template.
    pub start: usize,

    /// End position in template.
    pub end: usize,

    /// Optional slot kind from template.
    pub kind: Option<SlotKind>,
}

impl Template {
    /// Create a new template from content.
    ///
    /// # Arguments
    ///
    /// * `content` - The template content with AI slots
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::Template;
    ///
    /// let template = Template::new("<div>{{AI:content}}</div>");
    /// assert!(template.slots.contains_key("content"));
    /// ```
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        let slots = Self::parse_slots(&content);

        Self {
            content,
            name: String::from("unnamed"),
            slots,
            metadata: TemplateMetadata::default(),
        }
    }

    /// Load a template from a file.
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = tokio::fs::read_to_string(path).await?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();

        Ok(Self {
            name,
            slots: Self::parse_slots(&content),
            content,
            metadata: TemplateMetadata::default(),
        })
    }

    /// Set the template name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set template metadata.
    pub fn with_metadata(mut self, metadata: TemplateMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add a slot definition with a custom prompt.
    ///
    /// # Arguments
    ///
    /// * `name` - Slot name (must match a slot in the template)
    /// * `prompt` - The AI prompt for generating code
    pub fn with_slot(mut self, name: impl Into<String>, prompt: impl Into<String>) -> Self {
        let name = name.into();
        if let Some(slot) = self.slots.get_mut(&name) {
            slot.prompt = prompt.into();
        } else {
            self.slots.insert(name.clone(), Slot::new(name, prompt));
        }
        self
    }

    /// Configure a slot with detailed options.
    pub fn configure_slot(mut self, slot: Slot) -> Self {
        self.slots.insert(slot.name.clone(), slot);
        self
    }

    /// Parse slots from template content.
    fn parse_slots(content: &str) -> HashMap<String, Slot> {
        let re = get_slot_regex();
        let mut slots = HashMap::new();

        for cap in re.captures_iter(content) {
            let name = cap[1].to_string();
            let kind = cap.get(2).map(|m| Self::parse_kind(m.as_str()));

            let mut slot = Slot::new(&name, format!("Generate code for: {}", name));
            if let Some(k) = kind {
                slot = slot.with_kind(k);
            }
            slots.insert(name, slot);
        }

        slots
    }

    /// Parse slot kind from string.
    fn parse_kind(s: &str) -> SlotKind {
        match s.to_lowercase().as_str() {
            "raw" => SlotKind::Raw,
            "function" | "fn" => SlotKind::Function,
            "class" | "struct" => SlotKind::Class,
            "html" => SlotKind::Html,
            "css" => SlotKind::Css,
            "js" | "javascript" => SlotKind::JavaScript,
            "component" => SlotKind::Component,
            other => SlotKind::Custom(other.to_string()),
        }
    }

    /// Find all slot locations in the template content.
    fn find_locations(&self) -> Vec<SlotLocation> {
        let re = get_slot_regex();
        let mut locations = Vec::new();

        for cap in re.captures_iter(&self.content) {
            let full_match = cap.get(0).unwrap();
            locations.push(SlotLocation {
                name: cap[1].to_string(),
                start: full_match.start(),
                end: full_match.end(),
                kind: cap.get(2).map(|m| Self::parse_kind(m.as_str())),
            });
        }

        // Sort by position in reverse order for replacement
        locations.sort_by(|a, b| b.start.cmp(&a.start));
        locations
    }

    /// Render the template with provided code injections.
    ///
    /// # Arguments
    ///
    /// * `injections` - Map of slot names to generated code
    pub fn render(&self, injections: &HashMap<String, String>) -> Result<String> {
        let mut result = self.content.clone();
        let locations = self.find_locations();

        for loc in locations {
            let code = if let Some(code) = injections.get(&loc.name) {
                code.clone()
            } else if let Some(slot) = self.slots.get(&loc.name) {
                if slot.required {
                    return Err(AetherError::SlotNotFound(loc.name));
                }
                slot.default.clone().unwrap_or_default()
            } else {
                return Err(AetherError::SlotNotFound(loc.name));
            };

            result.replace_range(loc.start..loc.end, &code);
        }

        Ok(result)
    }

    /// Get a list of slot names.
    pub fn slot_names(&self) -> Vec<&str> {
        self.slots.keys().map(|s| s.as_str()).collect()
    }

    /// Check if template has unfilled required slots.
    pub fn validate(&self, injections: &HashMap<String, String>) -> Result<()> {
        for (name, slot) in &self.slots {
            if slot.required && !injections.contains_key(name) {
                return Err(AetherError::SlotNotFound(name.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slots() {
        let template = Template::new("Hello {{AI:greeting}} World {{AI:content:html}}");
        assert_eq!(template.slots.len(), 2);
        assert!(template.slots.contains_key("greeting"));
        assert!(template.slots.contains_key("content"));
    }

    #[test]
    fn test_render_template() {
        let template = Template::new("<div>{{AI:content}}</div>");
        let mut injections = HashMap::new();
        injections.insert("content".to_string(), "<p>Hello</p>".to_string());

        let result = template.render(&injections).unwrap();
        assert_eq!(result, "<div><p>Hello</p></div>");
    }

    #[test]
    fn test_slot_kind_parsing() {
        let template = Template::new("{{AI:func:function}} {{AI:style:css}}");
        assert_eq!(template.slots.get("func").unwrap().kind, SlotKind::Function);
        assert_eq!(template.slots.get("style").unwrap().kind, SlotKind::Css);
    }
}
