#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextEncoding {
    StandardEncoding,
    MacRomanEncoding,
    WinAnsiEncoding,
    PdfDocEncoding,
}

impl TextEncoding {
    pub fn encode(&self, text: &str) -> Vec<u8> {
        // For now, we'll use simple ASCII encoding
        // In a full implementation, this would handle different encodings properly
        text.bytes().collect()
    }
    
    pub fn decode(&self, data: &[u8]) -> String {
        // Simple ASCII decoding for now
        String::from_utf8_lossy(data).to_string()
    }
}