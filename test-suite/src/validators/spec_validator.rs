//! PDF Specification Validator
//! 
//! Validates PDFs against ISO 32000 specifications.

use crate::spec_compliance::{SpecificationTest, Pdf17ComplianceTester, Pdf20ComplianceTester};
use anyhow::Result;

/// Validator for PDF specification compliance
pub struct SpecValidator {
    /// PDF version to validate against
    version: PdfVersion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdfVersion {
    V1_7,
    V2_0,
}

impl SpecValidator {
    /// Create a new specification validator
    pub fn new(version: PdfVersion) -> Self {
        Self { version }
    }
    
    /// Validate a PDF file
    pub fn validate(&self, pdf: &[u8]) -> Result<ValidationReport> {
        let results = match self.version {
            PdfVersion::V1_7 => {
                let tester = Pdf17ComplianceTester;
                tester.test_all(pdf)
            }
            PdfVersion::V2_0 => {
                let tester = Pdf20ComplianceTester;
                tester.test_all(pdf)
            }
        };
        
        let passed = results.iter().all(|r| r.passed);
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.passed).count();
        
        Ok(ValidationReport {
            version: self.version,
            passed,
            total_tests,
            passed_tests,
            test_results: results,
        })
    }
}

/// Validation report
#[derive(Debug)]
pub struct ValidationReport {
    /// PDF version tested against
    pub version: PdfVersion,
    /// Overall pass/fail
    pub passed: bool,
    /// Total number of tests
    pub total_tests: usize,
    /// Number of passed tests
    pub passed_tests: usize,
    /// Individual test results
    pub test_results: Vec<crate::spec_compliance::TestResult>,
}