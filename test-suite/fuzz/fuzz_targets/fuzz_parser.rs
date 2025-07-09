#![no_main]

use libfuzzer_sys::fuzz_target;
use oxidize_pdf_core::parser::PdfReader;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Fuzz the PDF parser with arbitrary input
    let cursor = Cursor::new(data);
    
    // Try to parse as PDF - we expect this to fail gracefully on invalid input
    let _ = PdfReader::new(cursor);
    
    // Also try to parse specific parts if it looks like a PDF
    if data.starts_with(b"%PDF-") {
        // Try to extract version
        if let Some(version_end) = data[5..].iter().position(|&b| b == b'\r' || b == b'\n') {
            let version_bytes = &data[5..5 + version_end];
            let _ = std::str::from_utf8(version_bytes);
        }
    }
});