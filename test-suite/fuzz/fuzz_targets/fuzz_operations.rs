#![no_main]

use libfuzzer_sys::fuzz_target;
use oxidize_pdf_core::operations::{PageRange, parse_page_range};

fuzz_target!(|data: &[u8]| {
    // Fuzz page range parsing
    if let Ok(range_str) = std::str::from_utf8(data) {
        let _ = parse_page_range(range_str);
        
        // Try specific patterns
        if range_str.contains('-') || range_str.contains(',') {
            let _ = parse_page_range(range_str);
        }
    }
    
    // Fuzz PageRange operations
    if data.len() >= 8 {
        // Create page ranges from fuzzer input
        let start = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let end = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        
        // Limit to reasonable values
        let start = (start % 10000) + 1;
        let end = (end % 10000) + 1;
        
        if start <= end {
            let range = PageRange::Range(start, end);
            let _ = range.contains(start);
            let _ = range.contains(end);
            let _ = range.contains((start + end) / 2);
        }
    }
});