mod font;
mod encoding;
mod metrics;
mod flow;
mod extraction;

pub use font::{Font, FontFamily};
pub use encoding::TextEncoding;
pub use metrics::{measure_text, measure_char, split_into_words};
pub use flow::{TextFlowContext, TextAlign};
pub use extraction::{TextExtractor, ExtractionOptions, ExtractedText, TextFragment};

use crate::error::Result;
use std::fmt::Write;

#[derive(Clone)]
pub struct TextContext {
    operations: String,
    current_font: Font,
    font_size: f64,
    text_matrix: [f64; 6],
}

impl TextContext {
    pub fn new() -> Self {
        Self {
            operations: String::new(),
            current_font: Font::Helvetica,
            font_size: 12.0,
            text_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
    
    pub fn set_font(&mut self, font: Font, size: f64) -> &mut Self {
        self.current_font = font;
        self.font_size = size;
        self
    }
    
    pub fn at(&mut self, x: f64, y: f64) -> &mut Self {
        self.text_matrix[4] = x;
        self.text_matrix[5] = y;
        self
    }
    
    pub fn write(&mut self, text: &str) -> Result<&mut Self> {
        // Begin text object
        self.operations.push_str("BT\n");
        
        // Set font
        write!(&mut self.operations, "/{} {} Tf\n", 
               self.current_font.pdf_name(), self.font_size).unwrap();
        
        // Set text position
        write!(&mut self.operations, "{:.2} {:.2} Td\n", 
               self.text_matrix[4], self.text_matrix[5]).unwrap();
        
        // Encode text using WinAnsiEncoding
        let encoding = TextEncoding::WinAnsiEncoding;
        let encoded_bytes = encoding.encode(text);
        
        // Show text as a literal string
        self.operations.push('(');
        for &byte in &encoded_bytes {
            match byte {
                b'(' => self.operations.push_str("\\("),
                b')' => self.operations.push_str("\\)"),
                b'\\' => self.operations.push_str("\\\\"),
                b'\n' => self.operations.push_str("\\n"),
                b'\r' => self.operations.push_str("\\r"),
                b'\t' => self.operations.push_str("\\t"),
                // For bytes in the printable ASCII range, write as is
                0x20..=0x7E => self.operations.push(byte as char),
                // For other bytes, write as octal escape sequences
                _ => write!(&mut self.operations, "\\{:03o}", byte).unwrap(),
            }
        }
        self.operations.push_str(") Tj\n");
        
        // End text object
        self.operations.push_str("ET\n");
        
        Ok(self)
    }
    
    pub fn write_line(&mut self, text: &str) -> Result<&mut Self> {
        self.write(text)?;
        self.text_matrix[5] -= self.font_size * 1.2; // Move down for next line
        Ok(self)
    }
    
    pub fn set_character_spacing(&mut self, spacing: f64) -> &mut Self {
        write!(&mut self.operations, "{:.2} Tc\n", spacing).unwrap();
        self
    }
    
    pub fn set_word_spacing(&mut self, spacing: f64) -> &mut Self {
        write!(&mut self.operations, "{:.2} Tw\n", spacing).unwrap();
        self
    }
    
    pub fn set_horizontal_scaling(&mut self, scale: f64) -> &mut Self {
        write!(&mut self.operations, "{:.2} Tz\n", scale * 100.0).unwrap();
        self
    }
    
    pub fn set_leading(&mut self, leading: f64) -> &mut Self {
        write!(&mut self.operations, "{:.2} TL\n", leading).unwrap();
        self
    }
    
    pub fn set_text_rise(&mut self, rise: f64) -> &mut Self {
        write!(&mut self.operations, "{:.2} Ts\n", rise).unwrap();
        self
    }
    
    pub(crate) fn generate_operations(&self) -> Result<Vec<u8>> {
        Ok(self.operations.as_bytes().to_vec())
    }
}