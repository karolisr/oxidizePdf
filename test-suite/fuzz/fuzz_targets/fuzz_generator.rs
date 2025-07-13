#![no_main]

use libfuzzer_sys::fuzz_target;
use oxidize_pdf_test_suite::generators::test_pdf_builder::{TestPdfBuilder, PdfVersion};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    // Use fuzzer input to control PDF generation
    let mut builder = TestPdfBuilder::new();
    
    // Select PDF version based on first byte
    let version = match data[0] % 9 {
        0 => PdfVersion::V1_0,
        1 => PdfVersion::V1_1,
        2 => PdfVersion::V1_2,
        3 => PdfVersion::V1_3,
        4 => PdfVersion::V1_4,
        5 => PdfVersion::V1_5,
        6 => PdfVersion::V1_6,
        7 => PdfVersion::V1_7,
        _ => PdfVersion::V2_0,
    };
    builder = builder.with_version(version);
    
    // Add metadata based on fuzzer input
    if data.len() > 1 {
        let metadata_len = (data[1] as usize) % 50;
        if data.len() > metadata_len + 2 {
            if let Ok(title) = std::str::from_utf8(&data[2..2 + metadata_len]) {
                builder = builder.with_title(title);
            }
        }
    }
    
    // Add pages based on fuzzer input
    if data.len() > 10 {
        let page_count = (data[10] % 20) as usize;
        for i in 0..page_count {
            if data.len() > 11 + i {
                match data[11 + i] % 3 {
                    0 => builder.add_empty_page(612.0, 792.0),
                    1 => {
                        let text = format!("Page {}", i);
                        builder.add_text_page(&text, 12.0);
                    }
                    _ => builder.add_graphics_page(),
                }
            }
        }
    }
    
    // Generate PDF - should not panic
    let _ = builder.build();
});