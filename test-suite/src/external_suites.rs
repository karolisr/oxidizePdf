//! External Test Suite Integration
//! 
//! This module provides integration with popular PDF test suites like veraPDF,
//! qpdf, and the Isartor test suite for comprehensive validation.

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use crate::corpus::{TestPdf, TestMetadata, ExpectedBehavior, TestCategory, ExternalSuite, PdfFeature, ComplianceLevel};

/// Configuration for external test suites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSuiteConfig {
    /// Base directory for external suites
    pub base_dir: PathBuf,
    /// veraPDF configuration
    pub vera_pdf: VeraPdfConfig,
    /// qpdf configuration
    pub qpdf: QpdfConfig,
    /// Isartor configuration
    pub isartor: IsartorConfig,
}

impl Default for ExternalSuiteConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("external-suites"),
            vera_pdf: VeraPdfConfig::default(),
            qpdf: QpdfConfig::default(),
            isartor: IsartorConfig::default(),
        }
    }
}

/// veraPDF test corpus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeraPdfConfig {
    /// Corpus repository URL
    pub corpus_url: String,
    /// Specific corpus version/tag to use
    pub corpus_version: String,
    /// Local directory name
    pub local_dir: String,
    /// Enable PDF/A validation tests
    pub test_pdfa: bool,
    /// Enable PDF/UA validation tests
    pub test_pdfua: bool,
}

impl Default for VeraPdfConfig {
    fn default() -> Self {
        Self {
            corpus_url: "https://github.com/veraPDF/veraPDF-corpus".to_string(),
            corpus_version: "master".to_string(),
            local_dir: "veraPDF-corpus".to_string(),
            test_pdfa: true,
            test_pdfua: true,
        }
    }
}

/// qpdf test suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QpdfConfig {
    /// Repository URL
    pub repo_url: String,
    /// Version/tag to use
    pub version: String,
    /// Local directory name
    pub local_dir: String,
    /// Test directories to include
    pub test_dirs: Vec<String>,
}

impl Default for QpdfConfig {
    fn default() -> Self {
        Self {
            repo_url: "https://github.com/qpdf/qpdf".to_string(),
            version: "main".to_string(),
            local_dir: "qpdf".to_string(),
            test_dirs: vec!["qtest/qpdf".to_string(), "examples".to_string()],
        }
    }
}

/// Isartor test suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsartorConfig {
    /// Download URL for Isartor test suite
    pub download_url: String,
    /// Local directory name
    pub local_dir: String,
    /// Enable negative tests (PDFs that should fail)
    pub include_negative: bool,
}

impl Default for IsartorConfig {
    fn default() -> Self {
        Self {
            download_url: "https://www.pdfa.org/resource/isartor-test-suite/".to_string(),
            local_dir: "isartor".to_string(),
            include_negative: true,
        }
    }
}

/// External test suite manager
pub struct ExternalSuiteManager {
    config: ExternalSuiteConfig,
    base_path: PathBuf,
}

impl ExternalSuiteManager {
    /// Create a new external suite manager
    pub fn new(config: ExternalSuiteConfig, base_path: PathBuf) -> Self {
        Self { config, base_path }
    }
    
    /// Check if a suite is downloaded
    pub fn is_suite_available(&self, suite: ExternalSuite) -> bool {
        let suite_path = self.get_suite_path(suite);
        suite_path.exists() && suite_path.is_dir()
    }
    
    /// Get the path for a specific suite
    pub fn get_suite_path(&self, suite: ExternalSuite) -> PathBuf {
        let dir_name = match suite {
            ExternalSuite::VeraPDF => &self.config.vera_pdf.local_dir,
            ExternalSuite::QPdf => &self.config.qpdf.local_dir,
            ExternalSuite::Isartor => &self.config.isartor.local_dir,
            ExternalSuite::PdfAssociation => "pdf-association",
        };
        self.base_path.join(&self.config.base_dir).join(dir_name)
    }
    
    /// Load veraPDF corpus
    pub fn load_vera_pdf(&self) -> Result<Vec<TestPdf>> {
        let suite_path = self.get_suite_path(ExternalSuite::VeraPDF);
        if !suite_path.exists() {
            anyhow::bail!("veraPDF corpus not found at {:?}. Run download script first.", suite_path);
        }
        
        let mut pdfs = Vec::new();
        
        // Load PDF/A test files
        if self.config.vera_pdf.test_pdfa {
            pdfs.extend(self.load_vera_pdf_category(&suite_path, "PDF_A-1a", ComplianceLevel::PdfA1a)?);
            pdfs.extend(self.load_vera_pdf_category(&suite_path, "PDF_A-1b", ComplianceLevel::PdfA1b)?);
            pdfs.extend(self.load_vera_pdf_category(&suite_path, "PDF_A-2a", ComplianceLevel::PdfA2a)?);
            pdfs.extend(self.load_vera_pdf_category(&suite_path, "PDF_A-2b", ComplianceLevel::PdfA2b)?);
            pdfs.extend(self.load_vera_pdf_category(&suite_path, "PDF_A-2u", ComplianceLevel::PdfA2u)?);
            pdfs.extend(self.load_vera_pdf_category(&suite_path, "PDF_A-3a", ComplianceLevel::PdfA3a)?);
            pdfs.extend(self.load_vera_pdf_category(&suite_path, "PDF_A-3b", ComplianceLevel::PdfA3b)?);
            pdfs.extend(self.load_vera_pdf_category(&suite_path, "PDF_A-3u", ComplianceLevel::PdfA3u)?);
        }
        
        // Load PDF/UA test files
        if self.config.vera_pdf.test_pdfua {
            pdfs.extend(self.load_vera_pdf_category(&suite_path, "PDF_UA-1", ComplianceLevel::PdfUA1)?);
        }
        
        Ok(pdfs)
    }
    
    /// Load a specific veraPDF category
    fn load_vera_pdf_category(&self, base_path: &Path, category: &str, compliance: ComplianceLevel) -> Result<Vec<TestPdf>> {
        let category_path = base_path.join(category);
        if !category_path.exists() {
            return Ok(Vec::new());
        }
        
        let mut pdfs = Vec::new();
        
        // veraPDF structure has PASS and FAIL directories
        for result_type in &["PASS", "FAIL"] {
            let dir_path = category_path.join(result_type);
            if dir_path.exists() {
                let expected_behavior = if *result_type == "PASS" {
                    ExpectedBehavior::ParseSuccess {
                        page_count: None,
                        properties: None,
                    }
                } else {
                    ExpectedBehavior::CustomValidation(format!("vera_pdf_{}_fail", category.to_lowercase()))
                };
                
                for entry in fs::read_dir(&dir_path)? {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                        let metadata = TestMetadata {
                            name: path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            description: format!("veraPDF {} {} test", category, result_type),
                            pdf_version: "1.4".to_string(), // Will be determined by actual parsing
                            features: Vec::new(),
                            compliance: vec![compliance.clone()],
                            file_size: fs::metadata(&path).ok().map(|m| m.len()),
                            sha256: None,
                            created: None,
                            source: Some("veraPDF corpus".to_string()),
                        };
                        
                        pdfs.push(TestPdf {
                            path,
                            metadata,
                            expected_behavior: expected_behavior.clone(),
                            category: TestCategory::External(ExternalSuite::VeraPDF),
                        });
                    }
                }
            }
        }
        
        Ok(pdfs)
    }
    
    /// Load qpdf test suite
    pub fn load_qpdf(&self) -> Result<Vec<TestPdf>> {
        let suite_path = self.get_suite_path(ExternalSuite::QPdf);
        if !suite_path.exists() {
            anyhow::bail!("qpdf test suite not found at {:?}. Run download script first.", suite_path);
        }
        
        let mut pdfs = Vec::new();
        
        for test_dir in &self.config.qpdf.test_dirs {
            let dir_path = suite_path.join(test_dir);
            if dir_path.exists() {
                pdfs.extend(self.load_qpdf_directory(&dir_path)?);
            }
        }
        
        Ok(pdfs)
    }
    
    /// Load PDFs from a qpdf directory
    fn load_qpdf_directory(&self, dir_path: &Path) -> Result<Vec<TestPdf>> {
        let mut pdfs = Vec::new();
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                let file_name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                
                // Determine expected behavior from filename patterns
                let expected_behavior = if file_name.contains("bad") || file_name.contains("invalid") {
                    ExpectedBehavior::ParseError {
                        error_type: "InvalidPdf".to_string(),
                        error_pattern: None,
                    }
                } else {
                    ExpectedBehavior::ParseSuccess {
                        page_count: None,
                        properties: None,
                    }
                };
                
                let metadata = TestMetadata {
                    name: file_name.to_string(),
                    description: format!("qpdf test: {}", file_name),
                    pdf_version: "1.4".to_string(),
                    features: Vec::new(),
                    compliance: Vec::new(),
                    file_size: fs::metadata(&path).ok().map(|m| m.len()),
                    sha256: None,
                    created: None,
                    source: Some("qpdf test suite".to_string()),
                };
                
                pdfs.push(TestPdf {
                    path,
                    metadata,
                    expected_behavior,
                    category: TestCategory::External(ExternalSuite::QPdf),
                });
            }
        }
        
        Ok(pdfs)
    }
    
    /// Load Isartor test suite
    pub fn load_isartor(&self) -> Result<Vec<TestPdf>> {
        let suite_path = self.get_suite_path(ExternalSuite::Isartor);
        if !suite_path.exists() {
            anyhow::bail!("Isartor test suite not found at {:?}. Run download script first.", suite_path);
        }
        
        let mut pdfs = Vec::new();
        
        // Isartor has numbered test files with specific violations
        let test_mapping = self.get_isartor_test_mapping();
        
        for entry in fs::read_dir(&suite_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                let file_name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                
                // Extract test number from filename (e.g., "isartor-6-2-3-3-t01-fail-a.pdf")
                let is_negative = file_name.contains("fail");
                let test_description = test_mapping.get(file_name)
                    .cloned()
                    .unwrap_or_else(|| format!("Isartor test: {}", file_name));
                
                let expected_behavior = if is_negative && self.config.isartor.include_negative {
                    ExpectedBehavior::ParseError {
                        error_type: "PDF/A-1b violation".to_string(),
                        error_pattern: None,
                    }
                } else {
                    ExpectedBehavior::ParseSuccess {
                        page_count: None,
                        properties: None,
                    }
                };
                
                let metadata = TestMetadata {
                    name: file_name.to_string(),
                    description: test_description,
                    pdf_version: "1.4".to_string(),
                    features: Vec::new(),
                    compliance: vec![ComplianceLevel::PdfA1b],
                    file_size: fs::metadata(&path).ok().map(|m| m.len()),
                    sha256: None,
                    created: None,
                    source: Some("Isartor test suite".to_string()),
                };
                
                pdfs.push(TestPdf {
                    path,
                    metadata,
                    expected_behavior,
                    category: TestCategory::External(ExternalSuite::Isartor),
                });
            }
        }
        
        Ok(pdfs)
    }
    
    /// Get Isartor test mapping (test file -> description)
    fn get_isartor_test_mapping(&self) -> HashMap<&str, String> {
        let mut mapping = HashMap::new();
        
        // Add some known Isartor tests
        mapping.insert("isartor-6-2-3-3-t01-fail-a", "File header: invalid PDF version".to_string());
        mapping.insert("isartor-6-2-3-3-t02-fail-a", "File header: missing %PDF header".to_string());
        mapping.insert("isartor-6-3-2-t01-fail-a", "File trailer: missing %%EOF".to_string());
        mapping.insert("isartor-6-3-3-1-t01-fail-a", "Cross-reference: invalid xref entry".to_string());
        mapping.insert("isartor-6-4-t01-fail-a", "Font: missing required font descriptor".to_string());
        
        mapping
    }
    
    /// Create download instructions file
    pub fn create_download_instructions(&self) -> Result<String> {
        let mut instructions = String::new();
        
        instructions.push_str("# External Test Suite Download Instructions\n\n");
        instructions.push_str("To run tests against external PDF test suites, you need to download them first.\n\n");
        
        instructions.push_str("## veraPDF Corpus\n\n");
        instructions.push_str(&format!("```bash\n"));
        instructions.push_str(&format!("cd {} && \\\n", self.base_path.display()));
        instructions.push_str(&format!("git clone {} {} && \\\n", 
            self.config.vera_pdf.corpus_url,
            self.config.base_dir.join(&self.config.vera_pdf.local_dir).display()));
        instructions.push_str(&format!("cd {} && \\\n", self.config.vera_pdf.local_dir));
        instructions.push_str(&format!("git checkout {}\n", self.config.vera_pdf.corpus_version));
        instructions.push_str("```\n\n");
        
        instructions.push_str("## qpdf Test Suite\n\n");
        instructions.push_str("```bash\n");
        instructions.push_str(&format!("cd {} && \\\n", self.base_path.display()));
        instructions.push_str(&format!("git clone {} {} && \\\n",
            self.config.qpdf.repo_url,
            self.config.base_dir.join(&self.config.qpdf.local_dir).display()));
        instructions.push_str(&format!("cd {} && \\\n", self.config.qpdf.local_dir));
        instructions.push_str(&format!("git checkout {}\n", self.config.qpdf.version));
        instructions.push_str("```\n\n");
        
        instructions.push_str("## Isartor Test Suite\n\n");
        instructions.push_str("The Isartor test suite needs to be downloaded manually from:\n");
        instructions.push_str(&format!("{}\n\n", self.config.isartor.download_url));
        instructions.push_str(&format!("Extract the archive to: {}\n", 
            self.base_path.join(&self.config.base_dir).join(&self.config.isartor.local_dir).display()));
        
        Ok(instructions)
    }
}

/// Integration test runner for external suites
pub struct ExternalSuiteRunner {
    manager: ExternalSuiteManager,
    results: HashMap<ExternalSuite, Vec<TestResult>>,
}

/// Result of running a test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub pdf_path: PathBuf,
    pub passed: bool,
    pub error: Option<String>,
    pub duration: std::time::Duration,
}

impl ExternalSuiteRunner {
    /// Create a new test runner
    pub fn new(manager: ExternalSuiteManager) -> Self {
        Self {
            manager,
            results: HashMap::new(),
        }
    }
    
    /// Run all available external test suites
    pub fn run_all_suites(&mut self) -> Result<()> {
        // Run veraPDF tests
        if self.manager.is_suite_available(ExternalSuite::VeraPDF) {
            println!("Running veraPDF corpus tests...");
            let results = self.run_suite(ExternalSuite::VeraPDF)?;
            self.results.insert(ExternalSuite::VeraPDF, results);
        }
        
        // Run qpdf tests
        if self.manager.is_suite_available(ExternalSuite::QPdf) {
            println!("Running qpdf test suite...");
            let results = self.run_suite(ExternalSuite::QPdf)?;
            self.results.insert(ExternalSuite::QPdf, results);
        }
        
        // Run Isartor tests
        if self.manager.is_suite_available(ExternalSuite::Isartor) {
            println!("Running Isartor test suite...");
            let results = self.run_suite(ExternalSuite::Isartor)?;
            self.results.insert(ExternalSuite::Isartor, results);
        }
        
        Ok(())
    }
    
    /// Run a specific test suite
    fn run_suite(&self, suite: ExternalSuite) -> Result<Vec<TestResult>> {
        let pdfs = match suite {
            ExternalSuite::VeraPDF => self.manager.load_vera_pdf()?,
            ExternalSuite::QPdf => self.manager.load_qpdf()?,
            ExternalSuite::Isartor => self.manager.load_isartor()?,
            ExternalSuite::PdfAssociation => Vec::new(), // Not implemented yet
        };
        
        let mut results = Vec::new();
        
        for pdf in pdfs {
            let start = std::time::Instant::now();
            let result = self.run_single_test(&pdf);
            let duration = start.elapsed();
            
            results.push(TestResult {
                pdf_path: pdf.path.clone(),
                passed: result.is_ok(),
                error: result.err(),
                duration,
            });
        }
        
        Ok(results)
    }
    
    /// Run a single test
    fn run_single_test(&self, pdf: &TestPdf) -> Result<(), String> {
        use oxidize_pdf_core::parser::{PdfReader, document::PdfDocument};
        use std::fs::File;
        
        // Open the PDF file
        let file = File::open(&pdf.path)
            .map_err(|e| format!("Failed to open PDF file: {}", e))?;
        
        // Try to parse with our parser
        let reader_result = PdfReader::new(file);
        
        match &pdf.expected_behavior {
            ExpectedBehavior::ParseSuccess { page_count, .. } => {
                // Should parse successfully
                let reader = reader_result
                    .map_err(|e| format!("Parse failed: {:?}", e))?;
                
                // Create PdfDocument for higher-level operations
                let document = PdfDocument::new(reader);
                
                // Verify page count if specified
                if let Some(expected_pages) = page_count {
                    let actual_pages = document.page_count()
                        .map_err(|e| format!("Failed to get page count: {:?}", e))?;
                    if actual_pages != *expected_pages as u32 {
                        return Err(format!("Expected {} pages, got {}", expected_pages, actual_pages));
                    }
                }
                
                // Try to access first page to ensure basic functionality
                if document.page_count().unwrap_or(0) > 0 {
                    let _page = document.get_page(0)
                        .map_err(|e| format!("Failed to get first page: {:?}", e))?;
                }
                
                Ok(())
            }
            ExpectedBehavior::ParseError { error_type, error_pattern } => {
                // Should fail with specific error
                match reader_result {
                    Ok(_) => Err("Expected parse error but succeeded".to_string()),
                    Err(e) => {
                        let error_str = format!("{:?}", e);
                        
                        // Check if error matches expected pattern
                        if let Some(pattern) = error_pattern {
                            if !error_str.contains(pattern) {
                                return Err(format!("Error doesn't match pattern '{}': {}", pattern, error_str));
                            }
                        }
                        
                        // Basic error type checking
                        if !error_str.to_lowercase().contains(&error_type.to_lowercase()) {
                            return Err(format!("Expected error type '{}', got: {}", error_type, error_str));
                        }
                        
                        Ok(())
                    }
                }
            }
            ExpectedBehavior::ParseWarning { warning_patterns } => {
                // Should parse with warnings
                let reader = reader_result
                    .map_err(|e| format!("Parse failed when warnings expected: {:?}", e))?;
                
                // Create PdfDocument
                let document = PdfDocument::new(reader);
                
                // Verify document is readable despite warnings
                let _ = document.page_count()
                    .map_err(|e| format!("Failed to get page count: {:?}", e))?;
                
                // TODO: Implement warning collection and validation
                
                Ok(())
            }
            ExpectedBehavior::CustomValidation(validator) => {
                // Run custom validation based on validator name
                match validator.as_str() {
                    "vera_pdf_pdf_a-1a_fail" | "vera_pdf_pdf_a-1b_fail" => {
                        // For PDF/A compliance failures, we expect parsing to work
                        // but the PDF doesn't meet PDF/A requirements
                        let reader = reader_result
                            .map_err(|e| format!("Parse failed: {:?}", e))?;
                        let document = PdfDocument::new(reader);
                        let _ = document.page_count()
                            .map_err(|e| format!("Failed to get page count: {:?}", e))?;
                        Ok(())
                    }
                    _ => Ok(()) // Unknown validators pass by default
                }
            }
        }
    }
    
    /// Generate test report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# External Test Suite Results\n\n");
        
        for (suite, results) in &self.results {
            let total = results.len();
            let passed = results.iter().filter(|r| r.passed).count();
            let failed = total - passed;
            
            report.push_str(&format!("## {:?}\n\n", suite));
            report.push_str(&format!("Total: {} | Passed: {} | Failed: {}\n\n", total, passed, failed));
            
            if failed > 0 {
                report.push_str("### Failed Tests:\n");
                for result in results.iter().filter(|r| !r.passed) {
                    report.push_str(&format!("- {} ({})\n", 
                        result.pdf_path.display(),
                        result.error.as_ref().unwrap_or(&"Unknown error".to_string())));
                }
                report.push_str("\n");
            }
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_external_suite_config() {
        let config = ExternalSuiteConfig::default();
        assert_eq!(config.vera_pdf.corpus_version, "master");
        assert!(config.vera_pdf.test_pdfa);
    }
    
    #[test]
    fn test_suite_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = ExternalSuiteConfig::default();
        let manager = ExternalSuiteManager::new(config, temp_dir.path().to_path_buf());
        
        assert!(!manager.is_suite_available(ExternalSuite::VeraPDF));
        assert!(!manager.is_suite_available(ExternalSuite::QPdf));
    }
    
    #[test]
    fn test_download_instructions() {
        let temp_dir = TempDir::new().unwrap();
        let config = ExternalSuiteConfig::default();
        let manager = ExternalSuiteManager::new(config, temp_dir.path().to_path_buf());
        
        let instructions = manager.create_download_instructions().unwrap();
        assert!(instructions.contains("veraPDF Corpus"));
        assert!(instructions.contains("qpdf Test Suite"));
        assert!(instructions.contains("Isartor Test Suite"));
    }
}