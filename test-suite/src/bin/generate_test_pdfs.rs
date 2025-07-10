//! Generate Test PDFs
//!
//! This binary generates all test PDFs for the test suite.

use anyhow::Result;
use oxidize_pdf_test_suite::generators::minimal_pdfs;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("Generating test PDFs for oxidizePdf test suite...");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fixtures_dir = PathBuf::from(manifest_dir).join("fixtures");

    // Generate minimal PDFs
    let minimal_dir = fixtures_dir.join("valid/minimal");
    println!("Generating minimal PDFs in {:?}...", minimal_dir);
    minimal_pdfs::generate_all(&minimal_dir)?;

    // TODO: Generate other categories
    // - Standard PDFs
    // - Complex PDFs
    // - Edge cases
    // - Invalid PDFs

    println!("Test PDF generation complete!");
    Ok(())
}
