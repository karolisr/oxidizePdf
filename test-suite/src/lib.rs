//! Test Suite for oxidizePdf
//!
//! This module provides comprehensive testing infrastructure for validating
//! PDF parsing, generation, and manipulation against the PDF specification.

pub mod corpus;
pub mod external_suites;
pub mod generators;
pub mod parser_validation;
pub mod spec_compliance;
pub mod validators;

pub use corpus::{TestCategory, TestCorpus, TestMetadata, TestPdf};
pub use parser_validation::ParserValidator;
pub use spec_compliance::{test_compliance, SpecificationTest};

/// Common test utilities
pub mod utils {
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Get the path to the test fixtures directory
    pub fn fixtures_dir() -> PathBuf {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(manifest_dir).join("fixtures")
    }

    /// Read a test PDF file
    pub fn read_test_pdf<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<u8>> {
        let full_path = fixtures_dir().join(path);
        Ok(fs::read(full_path)?)
    }

    /// Create a temporary directory for test outputs
    pub fn create_test_output_dir() -> anyhow::Result<tempfile::TempDir> {
        Ok(tempfile::tempdir()?)
    }
}
