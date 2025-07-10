//! Content Stream Validator
//!
//! Validates PDF content streams for correctness.

use anyhow::{Context, Result};
use oxidize_pdf_core::parser::content::{ContentOperation, ContentParser};
use std::collections::HashSet;

/// Validator for PDF content streams
pub struct ContentValidator {
    /// Whether to allow deprecated operators
    allow_deprecated: bool,
    /// Maximum nesting depth for save/restore
    max_nesting_depth: usize,
}

impl ContentValidator {
    /// Create a new content validator
    pub fn new() -> Self {
        Self {
            allow_deprecated: true,
            max_nesting_depth: 28, // PDF spec recommends max 28
        }
    }

    /// Strict mode - disallow deprecated operators
    pub fn strict(mut self) -> Self {
        self.allow_deprecated = false;
        self
    }

    /// Validate a content stream
    pub fn validate(&self, content: &[u8]) -> Result<ContentValidationReport> {
        let operators = ContentParser::parse(content).context("Failed to parse content stream")?;

        let mut report = ContentValidationReport::new();
        let mut state = ValidationState::new();

        for (index, operator) in operators.iter().enumerate() {
            self.validate_operator(operator, &mut state, &mut report, index)?;
        }

        // Check for unclosed text objects
        if state.in_text_object {
            report.add_error("Unclosed text object (missing ET)");
        }

        // Check for unbalanced save/restore
        if state.graphics_state_depth > 0 {
            report.add_error(&format!(
                "Unbalanced graphics state: {} save operations without restore",
                state.graphics_state_depth
            ));
        }

        Ok(report)
    }

    /// Validate a single operator
    fn validate_operator(
        &self,
        operator: &ContentOperation,
        state: &mut ValidationState,
        report: &mut ContentValidationReport,
        index: usize,
    ) -> Result<()> {
        match operator {
            ContentOperation::BeginText => {
                if state.in_text_object {
                    report.add_error(&format!(
                        "Nested text object at operator {}: BT without ET",
                        index
                    ));
                }
                state.in_text_object = true;
            }

            ContentOperation::EndText => {
                if !state.in_text_object {
                    report.add_error(&format!("ET without BT at operator {}", index));
                }
                state.in_text_object = false;
            }

            ContentOperation::SaveGraphicsState => {
                state.graphics_state_depth += 1;
                if state.graphics_state_depth > self.max_nesting_depth {
                    report.add_error(&format!(
                        "Graphics state nesting too deep at operator {}: depth {}",
                        index, state.graphics_state_depth
                    ));
                }
            }

            ContentOperation::RestoreGraphicsState => {
                if state.graphics_state_depth == 0 {
                    report.add_error(&format!("Q without q at operator {}", index));
                } else {
                    state.graphics_state_depth -= 1;
                }
            }

            // Text operators that require being in text object
            ContentOperation::ShowText(_)
            | ContentOperation::ShowTextArray(_)
            | ContentOperation::NextLineShowText(_)
            | ContentOperation::SetSpacingNextLineShowText(_, _, _)
            | ContentOperation::MoveText(_, _)
            | ContentOperation::MoveTextSetLeading(_, _)
            | ContentOperation::SetTextMatrix(_, _, _, _, _, _)
            | ContentOperation::NextLine => {
                if !state.in_text_object {
                    report.add_warning(&format!(
                        "Text operator outside text object at operator {}: {:?}",
                        index, operator
                    ));
                }
            }

            // Deprecated operators
            ContentOperation::BeginCompatibility | ContentOperation::EndCompatibility => {
                if !self.allow_deprecated {
                    report
                        .add_warning(&format!("Deprecated operator at {}: {:?}", index, operator));
                }
            }

            _ => {
                // Other operators are generally valid
                report.operator_count += 1;
            }
        }

        // Track operator usage
        let op_name = format!("{:?}", operator)
            .split('(')
            .next()
            .unwrap_or("Unknown")
            .to_string();
        *report.operator_usage.entry(op_name).or_insert(0) += 1;

        Ok(())
    }
}

/// State tracked during validation
struct ValidationState {
    /// Whether we're currently in a text object
    in_text_object: bool,
    /// Current graphics state nesting depth
    graphics_state_depth: usize,
}

impl ValidationState {
    fn new() -> Self {
        Self {
            in_text_object: false,
            graphics_state_depth: 0,
        }
    }
}

/// Content validation report
#[derive(Debug)]
pub struct ContentValidationReport {
    /// Validation errors (things that are definitely wrong)
    pub errors: Vec<String>,
    /// Validation warnings (things that might be wrong)
    pub warnings: Vec<String>,
    /// Total operator count
    pub operator_count: usize,
    /// Operator usage statistics
    pub operator_usage: std::collections::HashMap<String, usize>,
}

impl ContentValidationReport {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            operator_count: 0,
            operator_usage: std::collections::HashMap::new(),
        }
    }

    fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for ContentValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_content_stream() {
        let content = b"BT /F1 12 Tf 100 700 Td (Hello) Tj ET";
        let validator = ContentValidator::new();
        let report = validator.validate(content).unwrap();
        assert!(report.is_valid());
    }

    #[test]
    fn test_unclosed_text_object() {
        let content = b"BT /F1 12 Tf 100 700 Td (Hello) Tj";
        let validator = ContentValidator::new();
        let report = validator.validate(content).unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.contains("Unclosed text object")));
    }

    #[test]
    fn test_unbalanced_graphics_state() {
        let content = b"q 1 0 0 1 50 50 cm";
        let validator = ContentValidator::new();
        let report = validator.validate(content).unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.contains("Unbalanced graphics state")));
    }
}
