//! External Test Suite Integration Tests
//!
//! These tests are ignored by default and only run when external test suites
//! are available. Run with: cargo test -- --ignored external

use oxidize_pdf_test_suite::{
    corpus::ExternalSuite,
    external_suites::{ExternalSuiteConfig, ExternalSuiteManager, ExternalSuiteRunner},
};
use std::env;
use std::path::PathBuf;

/// Get the path to external suites directory
fn external_suites_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("external-suites")
}

#[test]
#[ignore = "requires external test suites to be downloaded"]
fn test_vera_pdf_corpus() {
    let config = ExternalSuiteConfig::default();
    let manager = ExternalSuiteManager::new(config, PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    if !manager.is_suite_available(ExternalSuite::VeraPDF) {
        eprintln!("veraPDF corpus not available. Run download_external_suites.sh first.");
        return;
    }

    let pdfs = manager
        .load_vera_pdf()
        .expect("Failed to load veraPDF corpus");
    assert!(!pdfs.is_empty(), "veraPDF corpus should contain test PDFs");

    // Check that we have both PASS and FAIL tests
    let pass_count = pdfs
        .iter()
        .filter(|pdf| {
            matches!(
                &pdf.expected_behavior,
                oxidize_pdf_test_suite::corpus::ExpectedBehavior::ParseSuccess { .. }
            )
        })
        .count();
    let fail_count = pdfs.len() - pass_count;

    println!(
        "veraPDF corpus: {} total PDFs ({} pass, {} fail)",
        pdfs.len(),
        pass_count,
        fail_count
    );
    assert!(pass_count > 0, "Should have passing tests");
    assert!(fail_count > 0, "Should have failing tests");
}

#[test]
#[ignore = "requires external test suites to be downloaded"]
fn test_qpdf_suite() {
    let config = ExternalSuiteConfig::default();
    let manager = ExternalSuiteManager::new(config, PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    if !manager.is_suite_available(ExternalSuite::QPdf) {
        eprintln!("qpdf test suite not available. Run download_external_suites.sh first.");
        return;
    }

    let pdfs = manager.load_qpdf().expect("Failed to load qpdf test suite");
    assert!(!pdfs.is_empty(), "qpdf test suite should contain test PDFs");

    println!("qpdf test suite: {} PDFs loaded", pdfs.len());

    // Check for specific test categories
    let bad_pdfs = pdfs
        .iter()
        .filter(|pdf| {
            pdf.path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains("bad") || n.contains("invalid"))
                .unwrap_or(false)
        })
        .count();

    println!(
        "Found {} PDFs with 'bad' or 'invalid' in filename",
        bad_pdfs
    );
}

#[test]
#[ignore = "requires external test suites to be downloaded"]
fn test_isartor_suite() {
    let config = ExternalSuiteConfig::default();
    let manager = ExternalSuiteManager::new(config, PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    if !manager.is_suite_available(ExternalSuite::Isartor) {
        eprintln!("Isartor test suite not available. Please download manually.");
        return;
    }

    let pdfs = manager
        .load_isartor()
        .expect("Failed to load Isartor test suite");
    assert!(
        !pdfs.is_empty(),
        "Isartor test suite should contain test PDFs"
    );

    println!("Isartor test suite: {} PDFs loaded", pdfs.len());

    // All Isartor tests should be for PDF/A-1b compliance
    for pdf in &pdfs {
        assert!(
            pdf.metadata
                .compliance
                .iter()
                .any(|c| matches!(c, oxidize_pdf_test_suite::corpus::ComplianceLevel::PdfA1b)),
            "All Isartor tests should be PDF/A-1b compliance tests"
        );
    }
}

#[test]
#[ignore = "requires external test suites and working parser"]
fn test_run_all_external_suites() {
    let config = ExternalSuiteConfig::default();
    let manager = ExternalSuiteManager::new(config, PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    let mut runner = ExternalSuiteRunner::new(manager);

    // Run all available suites
    runner.run_all_suites().expect("Failed to run test suites");

    // Generate report
    let report = runner.generate_report();
    println!("\n{}", report);

    // Save report to file
    let report_path = external_suites_dir().join("test-results.md");
    std::fs::write(&report_path, &report).expect("Failed to write test report");
    println!("Test report saved to: {}", report_path.display());
}

#[test]
#[ignore = "requires external test suites"]
fn test_suite_availability() {
    let config = ExternalSuiteConfig::default();
    let manager = ExternalSuiteManager::new(config, PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    println!("\nExternal Test Suite Availability:");
    println!(
        "  veraPDF corpus: {}",
        if manager.is_suite_available(ExternalSuite::VeraPDF) {
            "✓ Available"
        } else {
            "✗ Not found"
        }
    );
    println!(
        "  qpdf test suite: {}",
        if manager.is_suite_available(ExternalSuite::QPdf) {
            "✓ Available"
        } else {
            "✗ Not found"
        }
    );
    println!(
        "  Isartor test suite: {}",
        if manager.is_suite_available(ExternalSuite::Isartor) {
            "✓ Available"
        } else {
            "✗ Not found"
        }
    );
    println!(
        "  PDF Association: {}",
        if manager.is_suite_available(ExternalSuite::PdfAssociation) {
            "✓ Available"
        } else {
            "✗ Not found"
        }
    );

    // Create download instructions if any suite is missing
    if !manager.is_suite_available(ExternalSuite::VeraPDF)
        || !manager.is_suite_available(ExternalSuite::QPdf)
        || !manager.is_suite_available(ExternalSuite::Isartor)
    {
        let instructions = manager
            .create_download_instructions()
            .expect("Failed to create download instructions");

        let instructions_path = external_suites_dir().join("DOWNLOAD_INSTRUCTIONS.md");
        std::fs::create_dir_all(&external_suites_dir()).ok();
        std::fs::write(&instructions_path, &instructions)
            .expect("Failed to write download instructions");

        println!(
            "\nDownload instructions saved to: {}",
            instructions_path.display()
        );
        println!("Run: ./test-suite/scripts/download_external_suites.sh");
    }
}

#[test]
#[ignore = "requires external test suites"]
fn test_specific_compliance_level() {
    use oxidize_pdf_test_suite::corpus::ComplianceLevel;

    let config = ExternalSuiteConfig::default();
    let manager = ExternalSuiteManager::new(config, PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    if !manager.is_suite_available(ExternalSuite::VeraPDF) {
        eprintln!("veraPDF corpus not available for compliance testing");
        return;
    }

    let pdfs = manager
        .load_vera_pdf()
        .expect("Failed to load veraPDF corpus");

    // Test PDF/A-1b compliance tests
    let pdfa1b_tests = pdfs
        .iter()
        .filter(|pdf| pdf.metadata.compliance.contains(&ComplianceLevel::PdfA1b))
        .count();

    println!("Found {} PDF/A-1b compliance tests", pdfa1b_tests);

    // Test PDF/A-2b compliance tests
    let pdfa2b_tests = pdfs
        .iter()
        .filter(|pdf| pdf.metadata.compliance.contains(&ComplianceLevel::PdfA2b))
        .count();

    println!("Found {} PDF/A-2b compliance tests", pdfa2b_tests);

    // Test PDF/UA-1 compliance tests
    let pdfua1_tests = pdfs
        .iter()
        .filter(|pdf| pdf.metadata.compliance.contains(&ComplianceLevel::PdfUA1))
        .count();

    println!("Found {} PDF/UA-1 compliance tests", pdfua1_tests);
}
