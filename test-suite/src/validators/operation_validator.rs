//! Operation Validator
//!
//! Validates PDF operations (split, merge, rotate) for correctness.

use anyhow::Result;
use oxidize_pdf::operations::{
    merge_pdf_files, rotate_all_pages, split_pdf, RotationAngle, SplitMode, SplitOptions,
};
use std::path::Path;
use tempfile::TempDir;

/// Validator for PDF operations
pub struct OperationValidator {
    /// Temporary directory for test files
    temp_dir: TempDir,
}

impl OperationValidator {
    /// Create a new operation validator
    pub fn new() -> Result<Self> {
        Ok(Self {
            temp_dir: tempfile::tempdir()?,
        })
    }

    /// Validate split operation
    pub fn validate_split(&self, pdf_path: &Path) -> Result<SplitValidationReport> {
        let mut report = SplitValidationReport::new();

        // Test single page split
        let options = SplitOptions {
            mode: SplitMode::SinglePages,
            output_pattern: "page_{}.pdf".to_string(),
            preserve_metadata: true,
            optimize: false,
        };

        let output_dir = self.temp_dir.path().join("split_single");
        std::fs::create_dir_all(&output_dir)?;

        match split_pdf(pdf_path, options) {
            Ok(files) => {
                report.single_page_split = true;
                report.output_files = files.len();

                // Verify each output file exists and is valid
                for file in files {
                    if !file.exists() {
                        report
                            .errors
                            .push(format!("Output file does not exist: {file:?}"));
                    }
                }
            }
            Err(e) => {
                report.errors.push(format!("Single page split failed: {e}"));
            }
        }

        // TODO: Test other split modes (chunks, ranges, split points)

        Ok(report)
    }

    /// Validate merge operation
    pub fn validate_merge(&self, pdf_paths: &[&Path]) -> Result<MergeValidationReport> {
        let mut report = MergeValidationReport::new();

        if pdf_paths.len() < 2 {
            report
                .errors
                .push("Need at least 2 PDFs to test merge".to_string());
            return Ok(report);
        }

        let output_path = self.temp_dir.path().join("merged.pdf");

        match merge_pdf_files(pdf_paths, &output_path) {
            Ok(()) => {
                report.merge_successful = true;

                // Verify output exists
                if !output_path.exists() {
                    report.errors.push("Merged file does not exist".to_string());
                }

                // TODO: Verify page count equals sum of input pages
                // TODO: Verify content preservation
            }
            Err(e) => {
                report.errors.push(format!("Merge failed: {e}"));
            }
        }

        Ok(report)
    }

    /// Validate rotate operation
    pub fn validate_rotate(&self, pdf_path: &Path) -> Result<RotateValidationReport> {
        let mut report = RotateValidationReport::new();

        // Test different rotation angles
        let angles = vec![
            RotationAngle::Clockwise90,
            RotationAngle::Rotate180,
            RotationAngle::Clockwise270,
        ];

        for angle in angles {
            let output_path = self
                .temp_dir
                .path()
                .join(format!("rotated_{}.pdf", angle.to_degrees()));

            match rotate_all_pages(pdf_path, &output_path, angle) {
                Ok(()) => {
                    report.rotations_tested.push(angle.to_degrees());

                    if !output_path.exists() {
                        report.errors.push(format!(
                            "Rotated file does not exist for {} degrees",
                            angle.to_degrees()
                        ));
                    }

                    // TODO: Verify rotation was applied correctly
                }
                Err(e) => {
                    report
                        .errors
                        .push(format!("Rotation {} failed: {}", angle.to_degrees(), e));
                }
            }
        }

        report.all_rotations_successful = report.errors.is_empty();

        Ok(report)
    }

    /// Validate round-trip operations (split then merge)
    pub fn validate_round_trip(&self, pdf_path: &Path) -> Result<RoundTripValidationReport> {
        let mut report = RoundTripValidationReport::new();

        // First split the PDF
        let split_options = SplitOptions {
            mode: SplitMode::SinglePages,
            output_pattern: self
                .temp_dir
                .path()
                .join("split_page_{}.pdf")
                .to_string_lossy()
                .to_string(),
            preserve_metadata: true,
            optimize: false,
        };

        let split_files = match split_pdf(pdf_path, split_options) {
            Ok(files) => files,
            Err(e) => {
                report.errors.push(format!("Split failed: {e}"));
                return Ok(report);
            }
        };

        report.split_successful = true;
        report.pages_split = split_files.len();

        // Then merge them back
        let merged_path = self.temp_dir.path().join("round_trip_merged.pdf");
        let split_paths: Vec<&Path> = split_files.iter().map(|p| p.as_path()).collect();

        match merge_pdf_files(&split_paths, &merged_path) {
            Ok(()) => {
                report.merge_successful = true;

                // TODO: Compare original with round-trip result
                // - Same page count
                // - Same page sizes
                // - Content preserved
            }
            Err(e) => {
                report.errors.push(format!("Merge failed: {e}"));
            }
        }

        Ok(report)
    }
}

/// Split operation validation report
#[derive(Debug)]
pub struct SplitValidationReport {
    pub single_page_split: bool,
    pub output_files: usize,
    pub errors: Vec<String>,
}

impl SplitValidationReport {
    fn new() -> Self {
        Self {
            single_page_split: false,
            output_files: 0,
            errors: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty() && self.single_page_split
    }
}

/// Merge operation validation report
#[derive(Debug)]
pub struct MergeValidationReport {
    pub merge_successful: bool,
    pub errors: Vec<String>,
}

impl MergeValidationReport {
    fn new() -> Self {
        Self {
            merge_successful: false,
            errors: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty() && self.merge_successful
    }
}

/// Rotate operation validation report
#[derive(Debug)]
pub struct RotateValidationReport {
    pub all_rotations_successful: bool,
    pub rotations_tested: Vec<i32>,
    pub errors: Vec<String>,
}

impl RotateValidationReport {
    fn new() -> Self {
        Self {
            all_rotations_successful: false,
            rotations_tested: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty() && self.all_rotations_successful
    }
}

/// Round-trip validation report
#[derive(Debug)]
pub struct RoundTripValidationReport {
    pub split_successful: bool,
    pub merge_successful: bool,
    pub pages_split: usize,
    pub errors: Vec<String>,
}

impl RoundTripValidationReport {
    fn new() -> Self {
        Self {
            split_successful: false,
            merge_successful: false,
            pages_split: 0,
            errors: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty() && self.split_successful && self.merge_successful
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = OperationValidator::new().unwrap();
        assert!(validator.temp_dir.path().exists());
    }
}
