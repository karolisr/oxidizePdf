#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextEncoding {
    StandardEncoding,
    MacRomanEncoding,
    WinAnsiEncoding,
    PdfDocEncoding,
}

impl TextEncoding {
    pub fn encode(&self, text: &str) -> Vec<u8> {
        match self {
            TextEncoding::StandardEncoding | TextEncoding::PdfDocEncoding => {
                // For now, use UTF-8 encoding
                text.bytes().collect()
            }
            TextEncoding::WinAnsiEncoding => {
                // Convert UTF-8 to Windows-1252
                let mut result = Vec::new();
                for ch in text.chars() {
                    match ch as u32 {
                        // ASCII range
                        0x00..=0x7F => result.push(ch as u8),
                        // Latin-1 Supplement that overlaps with Windows-1252
                        0xA0..=0xFF => result.push(ch as u8),
                        // Special mappings for Windows-1252
                        0x20AC => result.push(0x80), // Euro sign
                        0x201A => result.push(0x82), // Single low quotation mark
                        0x0192 => result.push(0x83), // Latin small letter f with hook
                        0x201E => result.push(0x84), // Double low quotation mark
                        0x2026 => result.push(0x85), // Horizontal ellipsis
                        0x2020 => result.push(0x86), // Dagger
                        0x2021 => result.push(0x87), // Double dagger
                        0x02C6 => result.push(0x88), // Circumflex accent
                        0x2030 => result.push(0x89), // Per mille sign
                        0x0160 => result.push(0x8A), // Latin capital letter S with caron
                        0x2039 => result.push(0x8B), // Single left angle quotation mark
                        0x0152 => result.push(0x8C), // Latin capital ligature OE
                        0x017D => result.push(0x8E), // Latin capital letter Z with caron
                        0x2018 => result.push(0x91), // Left single quotation mark
                        0x2019 => result.push(0x92), // Right single quotation mark
                        0x201C => result.push(0x93), // Left double quotation mark
                        0x201D => result.push(0x94), // Right double quotation mark
                        0x2022 => result.push(0x95), // Bullet
                        0x2013 => result.push(0x96), // En dash
                        0x2014 => result.push(0x97), // Em dash
                        0x02DC => result.push(0x98), // Small tilde
                        0x2122 => result.push(0x99), // Trade mark sign
                        0x0161 => result.push(0x9A), // Latin small letter s with caron
                        0x203A => result.push(0x9B), // Single right angle quotation mark
                        0x0153 => result.push(0x9C), // Latin small ligature oe
                        0x017E => result.push(0x9E), // Latin small letter z with caron
                        0x0178 => result.push(0x9F), // Latin capital letter Y with diaeresis
                        // Default: use question mark for unmapped characters
                        _ => result.push(b'?'),
                    }
                }
                result
            }
            TextEncoding::MacRomanEncoding => {
                // For now, use simple ASCII encoding
                text.bytes().collect()
            }
        }
    }
    
    pub fn decode(&self, data: &[u8]) -> String {
        match self {
            TextEncoding::StandardEncoding | TextEncoding::PdfDocEncoding => {
                // For now, assume UTF-8
                String::from_utf8_lossy(data).to_string()
            }
            TextEncoding::WinAnsiEncoding => {
                // Decode Windows-1252 to UTF-8
                let mut result = String::new();
                for &byte in data {
                    let ch = match byte {
                        // ASCII range
                        0x00..=0x7F => byte as char,
                        // Windows-1252 specific mappings
                        0x80 => '\u{20AC}', // Euro sign
                        0x82 => '\u{201A}', // Single low quotation mark
                        0x83 => '\u{0192}', // Latin small letter f with hook
                        0x84 => '\u{201E}', // Double low quotation mark
                        0x85 => '\u{2026}', // Horizontal ellipsis
                        0x86 => '\u{2020}', // Dagger
                        0x87 => '\u{2021}', // Double dagger
                        0x88 => '\u{02C6}', // Circumflex accent
                        0x89 => '\u{2030}', // Per mille sign
                        0x8A => '\u{0160}', // Latin capital letter S with caron
                        0x8B => '\u{2039}', // Single left angle quotation mark
                        0x8C => '\u{0152}', // Latin capital ligature OE
                        0x8E => '\u{017D}', // Latin capital letter Z with caron
                        0x91 => '\u{2018}', // Left single quotation mark
                        0x92 => '\u{2019}', // Right single quotation mark
                        0x93 => '\u{201C}', // Left double quotation mark
                        0x94 => '\u{201D}', // Right double quotation mark
                        0x95 => '\u{2022}', // Bullet
                        0x96 => '\u{2013}', // En dash
                        0x97 => '\u{2014}', // Em dash
                        0x98 => '\u{02DC}', // Small tilde
                        0x99 => '\u{2122}', // Trade mark sign
                        0x9A => '\u{0161}', // Latin small letter s with caron
                        0x9B => '\u{203A}', // Single right angle quotation mark
                        0x9C => '\u{0153}', // Latin small ligature oe
                        0x9E => '\u{017E}', // Latin small letter z with caron
                        0x9F => '\u{0178}', // Latin capital letter Y with diaeresis
                        // Latin-1 range that overlaps with Windows-1252
                        0xA0..=0xFF => char::from_u32(byte as u32).unwrap_or('?'),
                        // Undefined bytes
                        _ => '?',
                    };
                    result.push(ch);
                }
                result
            }
            TextEncoding::MacRomanEncoding => {
                // For now, use simple decoding
                String::from_utf8_lossy(data).to_string()
            }
        }
    }
}