#![no_main]

use libfuzzer_sys::fuzz_target;
use oxidize_pdf_core::parser::content::ContentParser;

fuzz_target!(|data: &[u8]| {
    // Fuzz the content stream parser
    let _ = ContentParser::parse(data);
    
    // Try parsing with different encodings
    if let Ok(text) = std::str::from_utf8(data) {
        // If it's valid UTF-8, try parsing operators
        for line in text.lines() {
            let _ = line.trim();
        }
    }
    
    // Try parsing specific operator patterns
    if data.len() > 2 {
        match &data[0..2] {
            b"BT" => {
                // Looks like text object start
                let _ = ContentParser::parse(data);
            }
            b"q " => {
                // Looks like save graphics state
                let _ = ContentParser::parse(data);
            }
            _ => {}
        }
    }
});