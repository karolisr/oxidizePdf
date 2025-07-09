//! Minimal PDF Generators
//! 
//! Generates minimal valid PDFs according to PDF specification.

use super::test_pdf_builder::{TestPdfBuilder, PdfVersion};
use std::path::Path;
use std::fs;
use anyhow::Result;

/// Generate all minimal test PDFs
pub fn generate_all<P: AsRef<Path>>(output_dir: P) -> Result<()> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;
    
    // Minimal PDF with just one empty page
    generate_minimal_empty(output_dir)?;
    
    // Minimal PDF with different versions
    generate_version_variants(output_dir)?;
    
    // Minimal PDF with single text
    generate_minimal_text(output_dir)?;
    
    // Minimal PDF with basic graphics
    generate_minimal_graphics(output_dir)?;
    
    // Minimal PDF with metadata
    generate_minimal_with_info(output_dir)?;
    
    // Minimal multi-page PDF
    generate_minimal_multipage(output_dir)?;
    
    Ok(())
}

/// Generate the absolute minimal valid PDF
fn generate_minimal_empty(output_dir: &Path) -> Result<()> {
    let pdf = TestPdfBuilder::minimal()
        .with_title("Minimal Empty PDF")
        .with_creator("oxidizePdf Test Suite")
        .build();
    
    let path = output_dir.join("minimal_empty.pdf");
    fs::write(&path, pdf)?;
    
    // Write metadata
    let metadata = r#"{
    "metadata": {
        "name": "minimal_empty",
        "description": "Absolute minimal valid PDF with one empty page",
        "pdf_version": "1.4",
        "features": [],
        "compliance": ["Pdf17"]
    },
    "expected_behavior": {
        "ParseSuccess": {
            "page_count": 1
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate minimal PDFs with different PDF versions
fn generate_version_variants(output_dir: &Path) -> Result<()> {
    let versions = vec![
        (PdfVersion::V1_0, "1.0"),
        (PdfVersion::V1_1, "1.1"),
        (PdfVersion::V1_2, "1.2"),
        (PdfVersion::V1_3, "1.3"),
        (PdfVersion::V1_4, "1.4"),
        (PdfVersion::V1_5, "1.5"),
        (PdfVersion::V1_6, "1.6"),
        (PdfVersion::V1_7, "1.7"),
        (PdfVersion::V2_0, "2.0"),
    ];
    
    for (version, version_str) in versions {
        let pdf = TestPdfBuilder::minimal()
            .with_version(version)
            .with_title(&format!("PDF Version {} Test", version_str))
            .build();
        
        let filename = format!("minimal_v{}.pdf", version_str.replace('.', "_"));
        let path = output_dir.join(&filename);
        fs::write(&path, pdf)?;
        
        let metadata = format!(r#"{{
    "metadata": {{
        "name": "minimal_v{}",
        "description": "Minimal PDF with version {}",
        "pdf_version": "{}",
        "features": [],
        "compliance": ["Pdf17"]
    }},
    "expected_behavior": {{
        "ParseSuccess": {{
            "page_count": 1
        }}
    }}
}}"#, version_str.replace('.', "_"), version_str, version_str);
        
        fs::write(path.with_extension("json"), metadata)?;
    }
    
    Ok(())
}

/// Generate minimal PDF with text
fn generate_minimal_text(output_dir: &Path) -> Result<()> {
    let mut builder = TestPdfBuilder::new()
        .with_title("Minimal Text PDF")
        .with_creator("oxidizePdf Test Suite");
    
    builder.add_text_page("Hello, World!", 12.0);
    
    let pdf = builder.build();
    let path = output_dir.join("minimal_text.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "minimal_text",
        "description": "Minimal PDF with simple text content",
        "pdf_version": "1.4",
        "features": ["Text"],
        "compliance": ["Pdf17"]
    },
    "expected_behavior": {
        "ParseSuccess": {
            "page_count": 1
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate minimal PDF with graphics
fn generate_minimal_graphics(output_dir: &Path) -> Result<()> {
    let mut builder = TestPdfBuilder::new()
        .with_title("Minimal Graphics PDF")
        .with_creator("oxidizePdf Test Suite");
    
    builder.add_graphics_page();
    
    let pdf = builder.build();
    let path = output_dir.join("minimal_graphics.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "minimal_graphics",
        "description": "Minimal PDF with basic graphics (rectangles)",
        "pdf_version": "1.4",
        "features": ["Graphics"],
        "compliance": ["Pdf17"]
    },
    "expected_behavior": {
        "ParseSuccess": {
            "page_count": 1
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate minimal PDF with full metadata
fn generate_minimal_with_info(output_dir: &Path) -> Result<()> {
    let pdf = TestPdfBuilder::minimal()
        .with_title("Test Document")
        .with_author("John Doe")
        .with_creator("oxidizePdf Test Suite")
        .with_producer("oxidizePdf v0.1.0")
        .with_info("Subject", "PDF Testing")
        .with_info("Keywords", "test, pdf, minimal")
        .build();
    
    let path = output_dir.join("minimal_with_info.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "minimal_with_info",
        "description": "Minimal PDF with complete info dictionary",
        "pdf_version": "1.4",
        "features": [],
        "compliance": ["Pdf17"]
    },
    "expected_behavior": {
        "ParseSuccess": {
            "page_count": 1,
            "properties": {
                "Title": "Test Document",
                "Author": "John Doe",
                "Creator": "oxidizePdf Test Suite",
                "Producer": "oxidizePdf v0.1.0",
                "Subject": "PDF Testing",
                "Keywords": "test, pdf, minimal"
            }
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate minimal multi-page PDF
fn generate_minimal_multipage(output_dir: &Path) -> Result<()> {
    let mut builder = TestPdfBuilder::new()
        .with_title("Minimal Multi-page PDF")
        .with_creator("oxidizePdf Test Suite");
    
    // Add 3 pages
    builder.add_empty_page(612.0, 792.0);
    builder.add_text_page("Page 2", 14.0);
    builder.add_graphics_page();
    
    let pdf = builder.build();
    let path = output_dir.join("minimal_multipage.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "minimal_multipage",
        "description": "Minimal PDF with multiple pages",
        "pdf_version": "1.4",
        "features": ["Text", "Graphics"],
        "compliance": ["Pdf17"]
    },
    "expected_behavior": {
        "ParseSuccess": {
            "page_count": 3
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;
    
    #[test]
    fn test_generate_minimal_pdfs() {
        let temp_dir = tempfile::tempdir().unwrap();
        generate_all(temp_dir.path()).unwrap();
        
        // Check that files were created
        assert!(temp_dir.path().join("minimal_empty.pdf").exists());
        assert!(temp_dir.path().join("minimal_empty.json").exists());
        assert!(temp_dir.path().join("minimal_text.pdf").exists());
        assert!(temp_dir.path().join("minimal_graphics.pdf").exists());
    }
}