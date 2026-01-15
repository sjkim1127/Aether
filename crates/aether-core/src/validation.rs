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

// ============================================================
// RustValidator - Uses rustc and rustfmt
// ============================================================

/// A validator that uses Rust-specific tools (rustc, rustfmt).
pub struct RustValidator;

impl Validator for RustValidator {
    fn validate(&self, kind: &SlotKind, code: &str) -> Result<ValidationResult> {
        match kind {
            SlotKind::Function | SlotKind::Class | SlotKind::Component => {
                let has_tests = code.contains("#[test]");

                let mut tmp_file = NamedTempFile::with_suffix(".rs")
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

                // Check syntax and compilation
                // Create temp output in same dir as source to avoid cross-drive issues on Windows
                let out_file = tmp_file.path().with_extension("rmeta");
                let output = Command::new("rustc")
                    .arg("--crate-type=lib")
                    .arg("--crate-name=aether_validation_check")
                    .arg("--emit=metadata")
                    .arg("-o")
                    .arg(&out_file)
                    .arg(tmp_file.path())
                    .output()
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;
                
                // Clean up output file
                let _ = std::fs::remove_file(&out_file);

                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    return Ok(ValidationResult::Invalid(format!("Rust Compilation Error:\n{}", err)));
                }

                // Run tests if present
                if has_tests {
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
                        let err = String::from_utf8_lossy(&test_run.stdout).to_string();
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
            SlotKind::Function | SlotKind::Class | SlotKind::Component => {
                let mut tmp_file = NamedTempFile::with_suffix(".rs")
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;
                
                tmp_file.write_all(code.as_bytes())
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

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

// ============================================================
// JsValidator - Uses node and prettier/eslint
// ============================================================

/// A validator that uses JavaScript/Node.js tools.
pub struct JsValidator;

impl Validator for JsValidator {
    fn validate(&self, kind: &SlotKind, code: &str) -> Result<ValidationResult> {
        match kind {
            SlotKind::JavaScript | SlotKind::Component => {
                let mut tmp_file = NamedTempFile::with_suffix(".js")
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;
                
                tmp_file.write_all(code.as_bytes())
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

                // Use node --check for syntax validation
                let output = Command::new("node")
                    .arg("--check")
                    .arg(tmp_file.path())
                    .output()
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    return Ok(ValidationResult::Invalid(format!("JavaScript Syntax Error:\n{}", err)));
                }

                Ok(ValidationResult::Valid)
            }
            _ => Ok(ValidationResult::Valid),
        }
    }

    fn format(&self, kind: &SlotKind, code: &str) -> Result<String> {
        match kind {
            SlotKind::JavaScript | SlotKind::Component => {
                // Try prettier first, fallback to original
                let output = Command::new("npx")
                    .arg("prettier")
                    .arg("--parser=babel")
                    .arg("--stdin-filepath=temp.js")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn();

                if let Ok(mut child) = output {
                    if let Some(ref mut stdin) = child.stdin {
                        let _ = stdin.write_all(code.as_bytes());
                    }
                    if let Ok(output) = child.wait_with_output() {
                        if output.status.success() {
                            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                        }
                    }
                }

                Ok(code.to_string())
            }
            _ => Ok(code.to_string()),
        }
    }
}

// ============================================================
// PythonValidator - Uses python and ruff
// ============================================================

/// A validator that uses Python tools.
pub struct PythonValidator;

impl Validator for PythonValidator {
    fn validate(&self, kind: &SlotKind, code: &str) -> Result<ValidationResult> {
        match kind {
            SlotKind::Function | SlotKind::Class => {
                let mut tmp_file = NamedTempFile::with_suffix(".py")
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;
                
                tmp_file.write_all(code.as_bytes())
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

                // Use python -m py_compile for syntax check
                let output = Command::new("python")
                    .arg("-m")
                    .arg("py_compile")
                    .arg(tmp_file.path())
                    .output()
                    .map_err(|e| crate::AetherError::InjectionError(e.to_string()))?;

                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    return Ok(ValidationResult::Invalid(format!("Python Syntax Error:\n{}", err)));
                }

                // Optional: Run ruff for linting
                let ruff_output = Command::new("ruff")
                    .arg("check")
                    .arg("--select=E,F") // Errors and Pyflakes only
                    .arg(tmp_file.path())
                    .output();

                if let Ok(out) = ruff_output {
                    if !out.status.success() {
                        let warnings = String::from_utf8_lossy(&out.stdout).to_string();
                        if !warnings.is_empty() {
                            // Return as invalid with lint warnings
                            return Ok(ValidationResult::Invalid(format!("Python Lint Issues:\n{}", warnings)));
                        }
                    }
                }

                Ok(ValidationResult::Valid)
            }
            _ => Ok(ValidationResult::Valid),
        }
    }

    fn format(&self, kind: &SlotKind, code: &str) -> Result<String> {
        match kind {
            SlotKind::Function | SlotKind::Class => {
                // Use ruff format (or black as fallback)
                let output = Command::new("ruff")
                    .arg("format")
                    .arg("--stdin-filename=temp.py")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn();

                if let Ok(mut child) = output {
                    if let Some(ref mut stdin) = child.stdin {
                        let _ = stdin.write_all(code.as_bytes());
                    }
                    if let Ok(output) = child.wait_with_output() {
                        if output.status.success() {
                            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                        }
                    }
                }

                Ok(code.to_string())
            }
            _ => Ok(code.to_string()),
        }
    }
}

// ============================================================
// MultiValidator - Auto-selects based on SlotKind
// ============================================================

/// A multi-language validator that auto-selects the appropriate validator.
pub struct MultiValidator {
    rust: RustValidator,
    js: JsValidator,
    python: PythonValidator,
}

impl Default for MultiValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiValidator {
    pub fn new() -> Self {
        Self {
            rust: RustValidator,
            js: JsValidator,
            python: PythonValidator,
        }
    }
}

impl Validator for MultiValidator {
    fn validate(&self, kind: &SlotKind, code: &str) -> Result<ValidationResult> {
        match kind {
            SlotKind::JavaScript => self.js.validate(kind, code),
            SlotKind::Html | SlotKind::Css => Ok(ValidationResult::Valid), // TODO: Add HTML/CSS validators
            SlotKind::Raw => Ok(ValidationResult::Valid),
            // Default to Rust for function/class/component (legacy behavior)
            _ => {
                // Heuristic: detect language from code patterns
                if code.contains("def ") || code.contains("import ") && code.contains(":") {
                    self.python.validate(kind, code)
                } else if code.contains("function ") || code.contains("const ") || code.contains("=>") {
                    self.js.validate(kind, code)
                } else {
                    self.rust.validate(kind, code)
                }
            }
        }
    }

    fn format(&self, kind: &SlotKind, code: &str) -> Result<String> {
        match kind {
            SlotKind::JavaScript => self.js.format(kind, code),
            SlotKind::Html | SlotKind::Css | SlotKind::Raw => Ok(code.to_string()),
            _ => {
                if code.contains("def ") || code.contains("import ") && code.contains(":") {
                    self.python.format(kind, code)
                } else if code.contains("function ") || code.contains("const ") || code.contains("=>") {
                    self.js.format(kind, code)
                } else {
                    self.rust.format(kind, code)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_validator_valid_code() {
        let validator = RustValidator;
        let code = "fn hello() -> i32 { 42 }";
        let result = validator.validate(&SlotKind::Function, code).unwrap();
        assert_eq!(result, ValidationResult::Valid);
    }

    #[test]
    fn test_multi_validator_detects_python() {
        let validator = MultiValidator::new();
        let code = "def hello():\n    return 42";
        // Should detect as Python and validate
        let result = validator.validate(&SlotKind::Function, code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multi_validator_detects_js() {
        let validator = MultiValidator::new();
        let code = "const hello = () => 42;";
        let result = validator.validate(&SlotKind::Function, code);
        assert!(result.is_ok());
    }
}
