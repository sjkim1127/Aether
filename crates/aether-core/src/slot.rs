//! Slot definitions for code injection points.
//!
//! Slots are placeholders in templates where AI-generated code will be injected.

use serde::{Deserialize, Serialize};

/// Represents a slot in a template where code can be injected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Slot {
    /// Unique identifier for this slot.
    pub name: String,

    /// The prompt or instruction for AI to generate code.
    pub prompt: String,

    /// Type of slot (determines generation behavior).
    pub kind: SlotKind,

    /// Optional constraints on generated code.
    pub constraints: Option<SlotConstraints>,

    /// Whether this slot is required.
    pub required: bool,

    /// Default value if AI generation fails.
    pub default: Option<String>,

    /// Specific temperature override for this slot (0.0 - 2.0).
    pub temperature: Option<f32>,
}

/// The kind of slot determines how code is generated.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SlotKind {
    /// Raw code injection (no wrapper).
    #[default]
    Raw,

    /// Function definition.
    Function,

    /// Class/struct definition.
    Class,

    /// HTML element.
    Html,

    /// CSS styles.
    Css,

    /// JavaScript code.
    JavaScript,

    /// Complete component (HTML + CSS + JS).
    Component,

    /// Custom kind with user-defined wrapper.
    Custom(String),
}

/// Constraints on generated code.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SlotConstraints {
    /// Maximum lines of code.
    pub max_lines: Option<usize>,

    /// Maximum characters.
    pub max_chars: Option<usize>,

    /// Required imports or dependencies.
    pub required_imports: Vec<String>,

    /// Forbidden patterns (regex).
    pub forbidden_patterns: Vec<String>,

    /// Language hint for code generation.
    pub language: Option<String>,

    /// TDD Test harness. This is the code that will be used to test the generated output.
    /// It should contain a placeholder like `{{CODE}}` where the generated code will be injected.
    pub test_harness: Option<String>,

    /// Command to execute the test harness (e.g., "cargo test", "node test.js").
    pub test_command: Option<String>,
}

impl Eq for Slot {}

impl std::hash::Hash for Slot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.prompt.hash(state);
        self.kind.hash(state);
        self.constraints.hash(state);
        self.required.hash(state);
        self.default.hash(state);
        if let Some(temp) = self.temperature {
            temp.to_bits().hash(state);
        }
    }
}

impl Slot {
    /// Create a new slot with the given name and prompt.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for this slot
    /// * `prompt` - The instruction for AI to generate code
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::Slot;
    ///
    /// let slot = Slot::new("button", "Create a submit button with hover effects");
    /// assert_eq!(slot.name, "button");
    /// ```
    pub fn new(name: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prompt: prompt.into(),
            kind: SlotKind::default(),
            constraints: None,
            required: true,
            default: None,
            temperature: None,
        }
    }

    /// Set a specific temperature for this slot.
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp.clamp(0.0, 2.0));
        self
    }

    /// Set the slot kind.
    pub fn with_kind(mut self, kind: SlotKind) -> Self {
        self.kind = kind;
        self
    }

    /// Set constraints on the generated code.
    pub fn with_constraints(mut self, constraints: SlotConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    /// Mark this slot as optional with a default value.
    pub fn optional(mut self, default: impl Into<String>) -> Self {
        self.required = false;
        self.default = Some(default.into());
        self
    }

    /// Validate the generated code against constraints.
    pub fn validate(&self, code: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Some(ref constraints) = self.constraints {
            // Check max lines
            if let Some(max) = constraints.max_lines {
                let lines = code.lines().count();
                if lines > max {
                    errors.push(format!("Code exceeds max lines: {} > {}", lines, max));
                }
            }

            // Check max chars
            if let Some(max) = constraints.max_chars {
                if code.len() > max {
                    errors.push(format!("Code exceeds max chars: {} > {}", code.len(), max));
                }
            }

            // Check forbidden patterns
            for pattern in &constraints.forbidden_patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(code) {
                        errors.push(format!("Code contains forbidden pattern: {}", pattern));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl SlotConstraints {
    /// Create new empty constraints.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum lines.
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
        self
    }

    /// Set maximum characters.
    pub fn max_chars(mut self, chars: usize) -> Self {
        self.max_chars = Some(chars);
        self
    }

    /// Set the target language.
    pub fn language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    /// Add a required import.
    pub fn require_import(mut self, import: impl Into<String>) -> Self {
        self.required_imports.push(import.into());
        self
    }

    /// Add a forbidden pattern.
    pub fn forbid_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.forbidden_patterns.push(pattern.into());
        self
    }

    /// Set a TDD test harness.
    pub fn test_harness(mut self, harness: impl Into<String>) -> Self {
        self.test_harness = Some(harness.into());
        self
    }

    /// Set a TDD test command.
    pub fn test_command(mut self, command: impl Into<String>) -> Self {
        self.test_command = Some(command.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_creation() {
        let slot = Slot::new("test", "Generate a test");
        assert_eq!(slot.name, "test");
        assert_eq!(slot.prompt, "Generate a test");
        assert!(slot.required);
    }

    #[test]
    fn test_slot_validation() {
        let slot = Slot::new("test", "")
            .with_constraints(SlotConstraints::new().max_lines(5));

        assert!(slot.validate("line1\nline2\nline3").is_ok());
        assert!(slot.validate("1\n2\n3\n4\n5\n6").is_err());
    }
}
