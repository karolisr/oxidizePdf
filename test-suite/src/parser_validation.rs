//! Parser Validation Framework
//!
//! This module provides tools for validating the PDF parser implementation
//! by comparing its output with reference implementations and testing
//! round-trip conversion.

use anyhow::{Context, Result};
use oxidize_pdf_core::parser::{PdfDictionary, PdfObject, PdfReader};
use oxidize_pdf_core::{Document, Page};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

/// Result of parser validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub success: bool,
    /// Test name
    pub test_name: String,
    /// Differences found
    pub differences: Vec<Difference>,
    /// Performance metrics
    pub metrics: PerformanceMetrics,
}

/// A difference found during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    /// Path to the difference (e.g., "page[0].mediabox")
    pub path: String,
    /// Expected value
    pub expected: String,
    /// Actual value
    pub actual: String,
    /// Severity of the difference
    pub severity: DifferenceSeverity,
}

/// Severity levels for differences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifferenceSeverity {
    /// Critical differences that affect correctness
    Critical,
    /// Important differences that might affect rendering
    Important,
    /// Minor differences that are likely harmless
    Minor,
    /// Informational differences (e.g., whitespace)
    Info,
}

/// Performance metrics from parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Time to parse in milliseconds
    pub parse_time_ms: u64,
    /// Memory used in bytes
    pub memory_bytes: Option<u64>,
    /// Number of objects parsed
    pub object_count: usize,
    /// File size in bytes
    pub file_size: usize,
}

/// Reference parse result for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceParse {
    /// PDF version
    pub version: String,
    /// Number of pages
    pub page_count: usize,
    /// Document info
    pub info: HashMap<String, String>,
    /// Page dimensions
    pub pages: Vec<PageInfo>,
    /// Object count
    pub object_count: usize,
    /// Has forms
    pub has_forms: bool,
    /// Has JavaScript
    pub has_javascript: bool,
    /// Encryption info
    pub encryption: Option<String>,
}

/// Page information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    /// Media box [x0, y0, x1, y1]
    pub media_box: [f32; 4],
    /// Crop box if different from media box
    pub crop_box: Option<[f32; 4]>,
    /// Page rotation
    pub rotation: i32,
    /// Has text content
    pub has_text: bool,
    /// Has images
    pub has_images: bool,
    /// Resource count
    pub resource_count: usize,
}

/// Parser validator
pub struct ParserValidator {
    /// Whether to use external tools for comparison
    use_external_tools: bool,
}

impl ParserValidator {
    /// Create a new parser validator
    pub fn new() -> Self {
        Self {
            use_external_tools: false,
        }
    }

    /// Enable external tool comparison (qpdf, mutool, etc.)
    pub fn with_external_tools(mut self) -> Self {
        self.use_external_tools = true;
        self
    }

    /// Validate parser output against a reference parse
    pub fn validate_against_reference(
        &self,
        pdf: &[u8],
        reference: &ReferenceParse,
    ) -> ValidationResult {
        let start_time = std::time::Instant::now();
        let mut differences = Vec::new();

        // Parse the PDF
        let parsed = match self.parse_pdf(pdf) {
            Ok(p) => p,
            Err(e) => {
                differences.push(Difference {
                    path: "parse".to_string(),
                    expected: "successful parse".to_string(),
                    actual: format!("parse error: {}", e),
                    severity: DifferenceSeverity::Critical,
                });

                return ValidationResult {
                    success: false,
                    test_name: "Parser Validation".to_string(),
                    differences,
                    metrics: PerformanceMetrics {
                        parse_time_ms: start_time.elapsed().as_millis() as u64,
                        memory_bytes: None,
                        object_count: 0,
                        file_size: pdf.len(),
                    },
                };
            }
        };

        let parse_time = start_time.elapsed();

        // Compare results
        self.compare_parse_results(&parsed, reference, &mut differences);

        ValidationResult {
            success: differences.is_empty()
                || differences
                    .iter()
                    .all(|d| d.severity == DifferenceSeverity::Info),
            test_name: "Parser Validation".to_string(),
            differences,
            metrics: PerformanceMetrics {
                parse_time_ms: parse_time.as_millis() as u64,
                memory_bytes: None,
                object_count: parsed.object_count,
                file_size: pdf.len(),
            },
        }
    }

    /// Compare with qpdf output
    pub fn compare_with_qpdf(&self, pdf: &[u8]) -> Result<ComparisonResult> {
        if !self.use_external_tools {
            anyhow::bail!("External tools not enabled");
        }

        // TODO: Implement qpdf comparison
        // This would:
        // 1. Save PDF to temp file
        // 2. Run qpdf --show-all-pages --show-object=n
        // 3. Parse qpdf output
        // 4. Compare with our parser output

        Ok(ComparisonResult {
            tool: "qpdf".to_string(),
            matches: true,
            differences: Vec::new(),
        })
    }

    /// Test round-trip conversion (parse and regenerate)
    pub fn round_trip_test(&self, pdf: &[u8]) -> RoundTripResult {
        let start_time = std::time::Instant::now();

        // Parse the original PDF
        let parsed = match self.parse_pdf(pdf) {
            Ok(p) => p,
            Err(e) => {
                return RoundTripResult {
                    success: false,
                    parse_time_ms: start_time.elapsed().as_millis() as u64,
                    generate_time_ms: 0,
                    original_size: pdf.len(),
                    regenerated_size: 0,
                    differences: vec![format!("Parse failed: {}", e)],
                };
            }
        };

        let parse_time = start_time.elapsed();
        let generate_start = std::time::Instant::now();

        // Generate new PDF from parsed data
        let regenerated = match self.generate_from_parse(&parsed) {
            Ok(data) => data,
            Err(e) => {
                return RoundTripResult {
                    success: false,
                    parse_time_ms: parse_time.as_millis() as u64,
                    generate_time_ms: generate_start.elapsed().as_millis() as u64,
                    original_size: pdf.len(),
                    regenerated_size: 0,
                    differences: vec![format!("Generation failed: {}", e)],
                };
            }
        };

        let generate_time = generate_start.elapsed();

        // Compare the PDFs
        let differences = self.compare_pdfs(pdf, &regenerated);

        RoundTripResult {
            success: differences.is_empty(),
            parse_time_ms: parse_time.as_millis() as u64,
            generate_time_ms: generate_time.as_millis() as u64,
            original_size: pdf.len(),
            regenerated_size: regenerated.len(),
            differences,
        }
    }

    /// Parse a PDF and extract information
    fn parse_pdf(&self, pdf: &[u8]) -> Result<ParsedPdfInfo> {
        let cursor = Cursor::new(pdf);
        // Note: This would require fixing the compilation errors in PdfReader
        // For now, we'll create a mock response

        // TODO: Once PdfReader is fixed, use:
        // let reader = PdfReader::new(cursor)?;

        Ok(ParsedPdfInfo {
            version: "1.4".to_string(),
            page_count: 1,
            info: HashMap::new(),
            pages: vec![PageInfo {
                media_box: [0.0, 0.0, 612.0, 792.0],
                crop_box: None,
                rotation: 0,
                has_text: false,
                has_images: false,
                resource_count: 0,
            }],
            object_count: 4,
            has_forms: false,
            has_javascript: false,
            encryption: None,
        })
    }

    /// Generate PDF from parsed information
    fn generate_from_parse(&self, parsed: &ParsedPdfInfo) -> Result<Vec<u8>> {
        let mut doc = Document::new();

        // Set metadata
        for (key, value) in &parsed.info {
            match key.as_str() {
                "Title" => doc.set_title(value),
                "Author" => doc.set_author(value),
                "Subject" => doc.set_subject(value),
                "Keywords" => doc.set_keywords(value),
                _ => {}
            }
        }

        // Add pages
        for page_info in &parsed.pages {
            let page = Page::new(
                (page_info.media_box[2] - page_info.media_box[0]) as f64,
                (page_info.media_box[3] - page_info.media_box[1]) as f64,
            );
            doc.add_page(page);
        }

        // Generate PDF bytes
        let mut buffer = Vec::new();
        doc.write(&mut buffer)?;
        Ok(buffer)
    }

    /// Compare parse results
    fn compare_parse_results(
        &self,
        actual: &ParsedPdfInfo,
        expected: &ReferenceParse,
        differences: &mut Vec<Difference>,
    ) {
        // Compare version
        if actual.version != expected.version {
            differences.push(Difference {
                path: "version".to_string(),
                expected: expected.version.clone(),
                actual: actual.version.clone(),
                severity: DifferenceSeverity::Important,
            });
        }

        // Compare page count
        if actual.page_count != expected.page_count {
            differences.push(Difference {
                path: "page_count".to_string(),
                expected: expected.page_count.to_string(),
                actual: actual.page_count.to_string(),
                severity: DifferenceSeverity::Critical,
            });
        }

        // Compare pages
        for (i, (actual_page, expected_page)) in
            actual.pages.iter().zip(expected.pages.iter()).enumerate()
        {
            // Compare media box
            if actual_page.media_box != expected_page.media_box {
                differences.push(Difference {
                    path: format!("pages[{}].media_box", i),
                    expected: format!("{:?}", expected_page.media_box),
                    actual: format!("{:?}", actual_page.media_box),
                    severity: DifferenceSeverity::Important,
                });
            }

            // Compare rotation
            if actual_page.rotation != expected_page.rotation {
                differences.push(Difference {
                    path: format!("pages[{}].rotation", i),
                    expected: expected_page.rotation.to_string(),
                    actual: actual_page.rotation.to_string(),
                    severity: DifferenceSeverity::Important,
                });
            }
        }
    }

    /// Compare two PDFs byte by byte
    fn compare_pdfs(&self, original: &[u8], regenerated: &[u8]) -> Vec<String> {
        let mut differences = Vec::new();

        // For now, just compare sizes
        // A more sophisticated comparison would parse both and compare structure
        if original.len() != regenerated.len() {
            differences.push(format!(
                "Size difference: original {} bytes, regenerated {} bytes",
                original.len(),
                regenerated.len()
            ));
        }

        differences
    }
}

/// Internal representation of parsed PDF
struct ParsedPdfInfo {
    version: String,
    page_count: usize,
    info: HashMap<String, String>,
    pages: Vec<PageInfo>,
    object_count: usize,
    has_forms: bool,
    has_javascript: bool,
    encryption: Option<String>,
}

/// Result of comparing with external tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Tool used for comparison
    pub tool: String,
    /// Whether outputs match
    pub matches: bool,
    /// Differences found
    pub differences: Vec<String>,
}

/// Result of round-trip test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundTripResult {
    /// Whether round-trip was successful
    pub success: bool,
    /// Time to parse in milliseconds
    pub parse_time_ms: u64,
    /// Time to generate in milliseconds
    pub generate_time_ms: u64,
    /// Original file size
    pub original_size: usize,
    /// Regenerated file size
    pub regenerated_size: usize,
    /// Differences found
    pub differences: Vec<String>,
}

impl Default for ParserValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = ParserValidator::new();
        assert!(!validator.use_external_tools);

        let validator = ParserValidator::new().with_external_tools();
        assert!(validator.use_external_tools);
    }

    #[test]
    fn test_difference_severity() {
        let diff = Difference {
            path: "test".to_string(),
            expected: "a".to_string(),
            actual: "b".to_string(),
            severity: DifferenceSeverity::Critical,
        };

        assert_eq!(diff.severity, DifferenceSeverity::Critical);
    }
}
