use crate::{Result, SlotKind};
use std::process::Command;
use std::io::Write;
use tempfile::NamedTempFile;

/// Result of a code validation check.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Validation passed.
    Valid,
    /// Validation failed with a specific error message.
    Invalid(String),
}

/// Trait for implementing code validators and formatters.
pub trait Validator: Send + Sync {
    /// Check if the code is valid according to the validator's rules.
    fn validate(&self, kind: &SlotKind, code: &str) -> Result<ValidationResult>;
    
    /// Format the code to comply with style guides.
    fn format(&self, kind: &SlotKind, code: &str) -> Result<String>;
}

/// A validator that uses Rust-specific tools (cargo check, rustfmt).
pub struct RustValidator;

impl Validator for RustValidator {
    fn validate(&self, kind: &SlotKind, code: &str) -> Result<ValidationResult> {
        // Only validate Rust-compatible slots
        match kind {
            SlotKind::Function | SlotKind::Class | SlotKind::Component => {
                // Wrap in a basic module structure if it looks like a snippet
                // If it contains tests, we will run them
                let has_tests = code.contains("#[test]");

                let mut tmp_file = NamedTempFile::new()
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;
                
                let wrapper = if has_tests {
                    code.to_string()
                } else {
                    format!(
                        "#[allow(dead_code, unused_variables, unused_imports)]\nmod validation_module {{\n{}\n}}",
                        code
                    )
                };
                
                tmp_file.write_all(wrapper.as_bytes())
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

                // 1. Check syntax and compilation first
                let output = Command::new("rustc")
                    .arg("--crate-type=lib")
                    .arg("--emit=metadata")
                    .arg("-o")
                    .arg("NUL") 
                    .arg(tmp_file.path())
                    .output()
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    return Ok(ValidationResult::Invalid(format!("Compilation Error:\n{}", err)));
                }

                // 2. If it has tests, run them!
                if has_tests {
                    // We need to compile as a test executable and run it
                    let test_exe = NamedTempFile::new()
                        .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;
                    
                    let test_compile = Command::new("rustc")
                        .arg("--test")
                        .arg("-o")
                        .arg(test_exe.path())
                        .arg(tmp_file.path())
                        .output()
                        .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

                    if !test_compile.status.success() {
                        let err = String::from_utf8_lossy(&test_compile.stderr).to_string();
                        return Ok(ValidationResult::Invalid(format!("Test Compilation Error:\n{}", err)));
                    }

                    let test_run = Command::new(test_exe.path())
                        .output()
                        .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

                    if !test_run.status.success() {
                        let err = String::from_utf8_lossy(&test_run.stdout).to_string(); // cargo test outputs to stdout
                        let stderr = String::from_utf8_lossy(&test_run.stderr).to_string();
                        return Ok(ValidationResult::Invalid(format!("Unit Test Failed:\n{}\n{}", err, stderr)));
                    }
                }

                Ok(ValidationResult::Valid)
            }
            _ => Ok(ValidationResult::Valid), 
        }
    }

    fn format(&self, kind: &SlotKind, code: &str) -> Result<String> {
        match kind {
            SlotKind::Function | SlotKind::Class | SlotKind::Component | SlotKind::JavaScript => {
                let mut tmp_file = NamedTempFile::new()
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;
                
                tmp_file.write_all(code.as_bytes())
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

                // Try rustfmt
                let output = Command::new("rustfmt")
                    .arg(tmp_file.path())
                    .output();

                if let Ok(out) = output {
                    if out.status.success() {
                        let formatted = std::fs::read_to_string(tmp_file.path())
                            .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;
                        return Ok(formatted);
                    }
                }
                
                Ok(code.to_string())
            }
            _ => Ok(code.to_string()),
        }
    }
}
