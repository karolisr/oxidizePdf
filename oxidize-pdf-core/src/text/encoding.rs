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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_encoding_variants() {
        let encodings = [
            TextEncoding::StandardEncoding,
            TextEncoding::MacRomanEncoding,
            TextEncoding::WinAnsiEncoding,
            TextEncoding::PdfDocEncoding,
        ];
        
        for encoding in &encodings {
            assert_eq!(*encoding, *encoding);
        }
        
        assert_ne!(TextEncoding::StandardEncoding, TextEncoding::WinAnsiEncoding);
    }
    
    #[test]
    fn test_standard_encoding_basic_ascii() {
        let encoding = TextEncoding::StandardEncoding;
        let text = "Hello World!";
        
        let encoded = encoding.encode(text);
        let decoded = encoding.decode(&encoded);
        
        assert_eq!(text, decoded);
    }
    
    #[test]
    fn test_pdf_doc_encoding_basic_ascii() {
        let encoding = TextEncoding::PdfDocEncoding;
        let text = "Hello World!";
        
        let encoded = encoding.encode(text);
        let decoded = encoding.decode(&encoded);
        
        assert_eq!(text, decoded);
    }
    
    #[test]
    fn test_mac_roman_encoding_basic_ascii() {
        let encoding = TextEncoding::MacRomanEncoding;
        let text = "Hello World!";
        
        let encoded = encoding.encode(text);
        let decoded = encoding.decode(&encoded);
        
        assert_eq!(text, decoded);
    }
    
    #[test]
    fn test_win_ansi_encoding_basic_ascii() {
        let encoding = TextEncoding::WinAnsiEncoding;
        let text = "Hello World!";
        
        let encoded = encoding.encode(text);
        let decoded = encoding.decode(&encoded);
        
        assert_eq!(text, decoded);
    }
    
    #[test]
    fn test_win_ansi_encoding_special_characters() {
        let encoding = TextEncoding::WinAnsiEncoding;
        
        // Test Euro sign
        let euro_text = "€";
        let encoded = encoding.encode(euro_text);
        assert_eq!(encoded, vec![0x80]);
        let decoded = encoding.decode(&encoded);
        assert_eq!(decoded, euro_text);
        
        // Test em dash
        let dash_text = "—";
        let encoded = encoding.encode(dash_text);
        assert_eq!(encoded, vec![0x97]);
        let decoded = encoding.decode(&encoded);
        assert_eq!(decoded, dash_text);
        
        // Test single low quotation mark
        let quote_text = "‚";
        let encoded = encoding.encode(quote_text);
        assert_eq!(encoded, vec![0x82]);
        let decoded = encoding.decode(&encoded);
        assert_eq!(decoded, quote_text);
    }
    
    #[test]
    fn test_win_ansi_encoding_latin_supplement() {
        let encoding = TextEncoding::WinAnsiEncoding;
        let text = "café";
        
        let encoded = encoding.encode(text);
        let decoded = encoding.decode(&encoded);
        
        assert_eq!(text, decoded);
    }
    
    #[test]
    fn test_win_ansi_encoding_unmapped_character() {
        let encoding = TextEncoding::WinAnsiEncoding;
        
        // Use a character that's not in Windows-1252
        let text = "❤"; // Heart emoji
        let encoded = encoding.encode(text);
        assert_eq!(encoded, vec![b'?']); // Should be replaced with ?
        
        let decoded = encoding.decode(&encoded);
        assert_eq!(decoded, "?");
    }
    
    #[test]
    fn test_win_ansi_encoding_round_trip_special_chars() {
        let encoding = TextEncoding::WinAnsiEncoding;
        
        let special_chars = [
            ("€", 0x80),  // Euro sign
            ("‚", 0x82),  // Single low quotation mark
            ("ƒ", 0x83),  // Latin small letter f with hook
            ("„", 0x84),  // Double low quotation mark
            ("…", 0x85),  // Horizontal ellipsis
            ("†", 0x86),  // Dagger
            ("‡", 0x87),  // Double dagger
            ("‰", 0x89),  // Per mille sign
            ("\u{2018}", 0x91),  // Left single quotation mark
            ("\u{2019}", 0x92),  // Right single quotation mark
            ("\u{201C}", 0x93),  // Left double quotation mark
            ("\u{201D}", 0x94),  // Right double quotation mark
            ("•", 0x95),  // Bullet
            ("–", 0x96),  // En dash
            ("—", 0x97),  // Em dash
            ("™", 0x99),  // Trade mark sign
        ];
        
        for (text, expected_byte) in &special_chars {
            let encoded = encoding.encode(text);
            assert_eq!(encoded, vec![*expected_byte], "Failed for character {}", text);
            
            let decoded = encoding.decode(&encoded);
            assert_eq!(decoded, *text, "Round trip failed for character {}", text);
        }
    }
    
    #[test]
    fn test_encoding_equality() {
        assert_eq!(TextEncoding::StandardEncoding, TextEncoding::StandardEncoding);
        assert_eq!(TextEncoding::WinAnsiEncoding, TextEncoding::WinAnsiEncoding);
        
        assert_ne!(TextEncoding::StandardEncoding, TextEncoding::WinAnsiEncoding);
        assert_ne!(TextEncoding::MacRomanEncoding, TextEncoding::PdfDocEncoding);
    }
    
    #[test]
    fn test_encoding_debug() {
        let encoding = TextEncoding::WinAnsiEncoding;
        let debug_str = format!("{:?}", encoding);
        assert_eq!(debug_str, "WinAnsiEncoding");
    }
    
    #[test]
    fn test_encoding_clone() {
        let encoding1 = TextEncoding::PdfDocEncoding;
        let encoding2 = encoding1;
        assert_eq!(encoding1, encoding2);
    }
    
    #[test]
    fn test_encoding_copy() {
        let encoding1 = TextEncoding::StandardEncoding;
        let encoding2 = encoding1; // Copy semantics
        assert_eq!(encoding1, encoding2);
        
        // Both variables should still be usable
        assert_eq!(encoding1, TextEncoding::StandardEncoding);
        assert_eq!(encoding2, TextEncoding::StandardEncoding);
    }
    
    #[test]
    fn test_empty_string_encoding() {
        for encoding in &[
            TextEncoding::StandardEncoding,
            TextEncoding::MacRomanEncoding,
            TextEncoding::WinAnsiEncoding,
            TextEncoding::PdfDocEncoding,
        ] {
            let encoded = encoding.encode("");
            assert!(encoded.is_empty());
            
            let decoded = encoding.decode(&[]);
            assert!(decoded.is_empty());
        }
    }
    
    #[test]
    fn test_win_ansi_decode_undefined_bytes() {
        let encoding = TextEncoding::WinAnsiEncoding;
        
        // Test some undefined bytes in Windows-1252 (0x81, 0x8D, 0x8F, 0x90, 0x9D)
        let undefined_bytes = [0x81, 0x8D, 0x8F, 0x90, 0x9D];
        
        for &byte in &undefined_bytes {
            let decoded = encoding.decode(&[byte]);
            assert_eq!(decoded, "?", "Undefined byte 0x{:02X} should decode to '?'", byte);
        }
    }
    
    #[test]
    fn test_win_ansi_ascii_range() {
        let encoding = TextEncoding::WinAnsiEncoding;
        
        // Test ASCII range (0x00-0x7F)
        for byte in 0x20..=0x7E { // Printable ASCII
            let text = char::from(byte).to_string();
            let encoded = encoding.encode(&text);
            assert_eq!(encoded, vec![byte]);
            
            let decoded = encoding.decode(&encoded);
            assert_eq!(decoded, text);
        }
    }
    
    #[test]
    fn test_win_ansi_latin1_overlap() {
        let encoding = TextEncoding::WinAnsiEncoding;
        
        // Test Latin-1 range that overlaps with Windows-1252 (0xA0-0xFF)
        let test_chars = "¡¢£¤¥¦§¨©ª«¬®¯°±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ";
        
        let encoded = encoding.encode(test_chars);
        let decoded = encoding.decode(&encoded);
        
        assert_eq!(decoded, test_chars);
    }
}