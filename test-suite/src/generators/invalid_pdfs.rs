//! Invalid PDF Generators
//! 
//! Generates deliberately invalid PDFs for error handling tests.

use std::path::Path;
use std::fs;
use anyhow::Result;

/// Generate all invalid test PDFs
pub fn generate_all<P: AsRef<Path>>(output_dir: P) -> Result<()> {
    let output_dir = output_dir.as_ref();
    
    // Corrupted PDFs
    let corrupted_dir = output_dir.join("corrupted");
    fs::create_dir_all(&corrupted_dir)?;
    generate_corrupted_pdfs(&corrupted_dir)?;
    
    // Malformed PDFs
    let malformed_dir = output_dir.join("malformed");
    fs::create_dir_all(&malformed_dir)?;
    generate_malformed_pdfs(&malformed_dir)?;
    
    // Security issue PDFs
    let security_dir = output_dir.join("security");
    fs::create_dir_all(&security_dir)?;
    generate_security_pdfs(&security_dir)?;
    
    Ok(())
}

/// Generate corrupted PDFs
fn generate_corrupted_pdfs(output_dir: &Path) -> Result<()> {
    // Truncated PDF
    generate_truncated_pdf(output_dir)?;
    
    // PDF with corrupted xref
    generate_corrupted_xref(output_dir)?;
    
    // PDF with invalid stream
    generate_corrupted_stream(output_dir)?;
    
    Ok(())
}

/// Generate malformed PDFs
fn generate_malformed_pdfs(output_dir: &Path) -> Result<()> {
    // Missing header
    generate_no_header(output_dir)?;
    
    // Invalid header
    generate_invalid_header(output_dir)?;
    
    // Missing EOF marker
    generate_no_eof(output_dir)?;
    
    // Circular references
    generate_circular_reference(output_dir)?;
    
    Ok(())
}

/// Generate PDFs with security issues
fn generate_security_pdfs(output_dir: &Path) -> Result<()> {
    // JavaScript injection attempt
    generate_javascript_injection(output_dir)?;
    
    // Excessive nesting
    generate_excessive_nesting(output_dir)?;
    
    Ok(())
}

/// Generate truncated PDF
fn generate_truncated_pdf(output_dir: &Path) -> Result<()> {
    let pdf = b"%PDF-1.4\n\
                %\xE2\xE3\xCF\xD3\n\
                1 0 obj\n\
                << /Type /Catalog /Pages 2 0 R >>\n\
                endobj\n\
                2 0 obj\n\
                << /Type /Pages /Kids [3 0 R] /Count 1 >>\n\
                endobj\n\
                3 0 obj\n\
                << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\n\
                endobj\n\
                xref\n\
                0 4\n\
                0000000000 655";  // Truncated here
    
    let path = output_dir.join("truncated.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "truncated",
        "description": "PDF file truncated in the middle of xref table",
        "pdf_version": "1.4",
        "features": [],
        "compliance": []
    },
    "expected_behavior": {
        "ParseError": {
            "error_type": "UnexpectedEof",
            "error_pattern": "unexpected end of file"
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate PDF with corrupted xref
fn generate_corrupted_xref(output_dir: &Path) -> Result<()> {
    let pdf = b"%PDF-1.4\n\
                %\xE2\xE3\xCF\xD3\n\
                1 0 obj\n\
                << /Type /Catalog /Pages 2 0 R >>\n\
                endobj\n\
                2 0 obj\n\
                << /Type /Pages /Kids [3 0 R] /Count 1 >>\n\
                endobj\n\
                3 0 obj\n\
                << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\n\
                endobj\n\
                xref\n\
                0 4\n\
                0000000000 65535 f \n\
                0000000015 XXXXX n \n\
                0000000068 00000 n \n\
                0000000125 00000 n \n\
                trailer\n\
                << /Size 4 /Root 1 0 R >>\n\
                startxref\n\
                229\n\
                %%EOF";
    
    let path = output_dir.join("corrupted_xref.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "corrupted_xref",
        "description": "PDF with invalid characters in xref table",
        "pdf_version": "1.4",
        "features": [],
        "compliance": []
    },
    "expected_behavior": {
        "ParseError": {
            "error_type": "InvalidXRef",
            "error_pattern": "invalid.*xref"
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate PDF with corrupted stream
fn generate_corrupted_stream(output_dir: &Path) -> Result<()> {
    let pdf = b"%PDF-1.4\n\
                1 0 obj\n\
                << /Type /Catalog /Pages 2 0 R >>\n\
                endobj\n\
                2 0 obj\n\
                << /Type /Pages /Kids [3 0 R] /Count 1 >>\n\
                endobj\n\
                3 0 obj\n\
                << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\n\
                endobj\n\
                4 0 obj\n\
                << /Length 100 /Filter /FlateDecode >>\n\
                stream\n\
                This is not valid compressed data!\n\
                endstream\n\
                endobj\n\
                xref\n\
                0 5\n\
                0000000000 65535 f \n\
                0000000009 00000 n \n\
                0000000062 00000 n \n\
                0000000119 00000 n \n\
                0000000217 00000 n \n\
                trailer\n\
                << /Size 5 /Root 1 0 R >>\n\
                startxref\n\
                337\n\
                %%EOF";
    
    let path = output_dir.join("corrupted_stream.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "corrupted_stream",
        "description": "PDF with invalid compressed stream data",
        "pdf_version": "1.4",
        "features": [],
        "compliance": []
    },
    "expected_behavior": {
        "ParseError": {
            "error_type": "StreamDecodeError",
            "error_pattern": "failed to decode.*stream"
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate PDF without header
fn generate_no_header(output_dir: &Path) -> Result<()> {
    let pdf = b"1 0 obj\n\
                << /Type /Catalog /Pages 2 0 R >>\n\
                endobj\n\
                2 0 obj\n\
                << /Type /Pages /Kids [] /Count 0 >>\n\
                endobj\n\
                xref\n\
                0 3\n\
                0000000000 65535 f \n\
                0000000000 00000 n \n\
                0000000053 00000 n \n\
                trailer\n\
                << /Size 3 /Root 1 0 R >>\n\
                startxref\n\
                108\n\
                %%EOF";
    
    let path = output_dir.join("no_header.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "no_header",
        "description": "PDF file missing the %PDF header",
        "pdf_version": "unknown",
        "features": [],
        "compliance": []
    },
    "expected_behavior": {
        "ParseError": {
            "error_type": "InvalidHeader",
            "error_pattern": "invalid.*header|missing.*header"
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate PDF with invalid header
fn generate_invalid_header(output_dir: &Path) -> Result<()> {
    let pdf = b"%PDF-9.9\n\
                1 0 obj\n\
                << /Type /Catalog /Pages 2 0 R >>\n\
                endobj\n\
                %%EOF";
    
    let path = output_dir.join("invalid_header.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "invalid_header",
        "description": "PDF with invalid version number in header",
        "pdf_version": "9.9",
        "features": [],
        "compliance": []
    },
    "expected_behavior": {
        "ParseError": {
            "error_type": "UnsupportedVersion",
            "error_pattern": "unsupported.*version|invalid.*version"
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate PDF without EOF marker
fn generate_no_eof(output_dir: &Path) -> Result<()> {
    let pdf = b"%PDF-1.4\n\
                1 0 obj\n\
                << /Type /Catalog /Pages 2 0 R >>\n\
                endobj\n\
                2 0 obj\n\
                << /Type /Pages /Kids [] /Count 0 >>\n\
                endobj\n\
                xref\n\
                0 3\n\
                0000000000 65535 f \n\
                0000000009 00000 n \n\
                0000000062 00000 n \n\
                trailer\n\
                << /Size 3 /Root 1 0 R >>\n\
                startxref\n\
                117";  // Missing %%EOF
    
    let path = output_dir.join("no_eof.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "no_eof",
        "description": "PDF file missing the %%EOF marker",
        "pdf_version": "1.4",
        "features": [],
        "compliance": []
    },
    "expected_behavior": {
        "ParseWarning": {
            "warning_patterns": ["missing.*EOF", "no.*EOF.*marker"]
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate PDF with circular reference
fn generate_circular_reference(output_dir: &Path) -> Result<()> {
    let pdf = b"%PDF-1.4\n\
                1 0 obj\n\
                << /Type /Catalog /Pages 2 0 R >>\n\
                endobj\n\
                2 0 obj\n\
                << /Type /Pages /Kids [3 0 R] /Count 1 /Parent 3 0 R >>\n\
                endobj\n\
                3 0 obj\n\
                << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Kids [2 0 R] >>\n\
                endobj\n\
                xref\n\
                0 4\n\
                0000000000 65535 f \n\
                0000000009 00000 n \n\
                0000000062 00000 n \n\
                0000000139 00000 n \n\
                trailer\n\
                << /Size 4 /Root 1 0 R >>\n\
                startxref\n\
                232\n\
                %%EOF";
    
    let path = output_dir.join("circular_reference.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "circular_reference",
        "description": "PDF with circular object references",
        "pdf_version": "1.4",
        "features": [],
        "compliance": []
    },
    "expected_behavior": {
        "ParseError": {
            "error_type": "CircularReference",
            "error_pattern": "circular.*reference"
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate PDF with JavaScript injection attempt
fn generate_javascript_injection(output_dir: &Path) -> Result<()> {
    let pdf = b"%PDF-1.4\n\
                1 0 obj\n\
                << /Type /Catalog /Pages 2 0 R /Names << /JavaScript 3 0 R >> >>\n\
                endobj\n\
                2 0 obj\n\
                << /Type /Pages /Kids [] /Count 0 >>\n\
                endobj\n\
                3 0 obj\n\
                << /Names [(Test) 4 0 R] >>\n\
                endobj\n\
                4 0 obj\n\
                << /S /JavaScript /JS (app.alert('XSS Test');) >>\n\
                endobj\n\
                xref\n\
                0 5\n\
                0000000000 65535 f \n\
                0000000009 00000 n \n\
                0000000089 00000 n \n\
                0000000142 00000 n \n\
                0000000182 00000 n \n\
                trailer\n\
                << /Size 5 /Root 1 0 R >>\n\
                startxref\n\
                243\n\
                %%EOF";
    
    let path = output_dir.join("javascript_injection.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "javascript_injection",
        "description": "PDF with embedded JavaScript code",
        "pdf_version": "1.4",
        "features": ["JavaScript"],
        "compliance": []
    },
    "expected_behavior": {
        "ParseWarning": {
            "warning_patterns": ["JavaScript.*detected", "potential.*security.*risk"]
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}

/// Generate PDF with excessive nesting
fn generate_excessive_nesting(output_dir: &Path) -> Result<()> {
    let mut pdf = b"%PDF-1.4\n1 0 obj\n<<".to_vec();
    
    // Create deeply nested dictionaries
    for _ in 0..100 {
        pdf.extend_from_slice(b" /Dict <<");
    }
    pdf.extend_from_slice(b" /Type /Test");
    for _ in 0..100 {
        pdf.extend_from_slice(b" >>");
    }
    
    pdf.extend_from_slice(b">>\nendobj\nxref\n0 2\n\
                            0000000000 65535 f \n\
                            0000000009 00000 n \n\
                            trailer\n<< /Size 2 /Root 1 0 R >>\n\
                            startxref\n420\n%%EOF");
    
    let path = output_dir.join("excessive_nesting.pdf");
    fs::write(&path, pdf)?;
    
    let metadata = r#"{
    "metadata": {
        "name": "excessive_nesting",
        "description": "PDF with excessively nested dictionaries",
        "pdf_version": "1.4",
        "features": [],
        "compliance": []
    },
    "expected_behavior": {
        "ParseError": {
            "error_type": "ExcessiveNesting",
            "error_pattern": "nesting.*too.*deep|stack.*overflow"
        }
    }
}"#;
    
    fs::write(path.with_extension("json"), metadata)?;
    Ok(())
}