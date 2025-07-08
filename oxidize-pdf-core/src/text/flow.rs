use crate::text::{Font, measure_text, split_into_words};
use crate::page::Margins;
use crate::error::Result;
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justified,
}

pub struct TextFlowContext {
    operations: String,
    current_font: Font,
    font_size: f64,
    line_height: f64,
    cursor_x: f64,
    cursor_y: f64,
    alignment: TextAlign,
    page_width: f64,
    page_height: f64,
    margins: Margins,
}

impl TextFlowContext {
    pub fn new(page_width: f64, page_height: f64, margins: Margins) -> Self {
        Self {
            operations: String::new(),
            current_font: Font::Helvetica,
            font_size: 12.0,
            line_height: 1.2,
            cursor_x: margins.left,
            cursor_y: page_height - margins.top,
            alignment: TextAlign::Left,
            page_width,
            page_height,
            margins,
        }
    }
    
    pub fn set_font(&mut self, font: Font, size: f64) -> &mut Self {
        self.current_font = font;
        self.font_size = size;
        self
    }
    
    pub fn set_line_height(&mut self, multiplier: f64) -> &mut Self {
        self.line_height = multiplier;
        self
    }
    
    pub fn set_alignment(&mut self, alignment: TextAlign) -> &mut Self {
        self.alignment = alignment;
        self
    }
    
    pub fn at(&mut self, x: f64, y: f64) -> &mut Self {
        self.cursor_x = x;
        self.cursor_y = y;
        self
    }
    
    pub fn content_width(&self) -> f64 {
        self.page_width - self.margins.left - self.margins.right
    }
    
    pub fn write_wrapped(&mut self, text: &str) -> Result<&mut Self> {
        let content_width = self.content_width();
        
        // Split text into words
        let words = split_into_words(text);
        let mut lines: Vec<Vec<&str>> = Vec::new();
        let mut current_line: Vec<&str> = Vec::new();
        let mut current_width = 0.0;
        
        // Build lines based on width constraints
        for word in words {
            let word_width = measure_text(word, self.current_font, self.font_size);
            
            // Check if we need to start a new line
            if !current_line.is_empty() && current_width + word_width > content_width {
                lines.push(current_line);
                current_line = vec![word];
                current_width = word_width;
            } else {
                current_line.push(word);
                current_width += word_width;
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        // Render each line
        for (i, line) in lines.iter().enumerate() {
            let line_text = line.join("");
            let line_width = measure_text(&line_text, self.current_font, self.font_size);
            
            // Calculate x position based on alignment
            let x = match self.alignment {
                TextAlign::Left => self.margins.left,
                TextAlign::Right => self.page_width - self.margins.right - line_width,
                TextAlign::Center => self.margins.left + (content_width - line_width) / 2.0,
                TextAlign::Justified => {
                    if i < lines.len() - 1 && line.len() > 1 {
                        // We'll handle justification below
                        self.margins.left
                    } else {
                        self.margins.left
                    }
                }
            };
            
            // Begin text object
            self.operations.push_str("BT\n");
            
            // Set font
            write!(&mut self.operations, "/{} {} Tf\n", 
                   self.current_font.pdf_name(), self.font_size).unwrap();
            
            // Set text position
            write!(&mut self.operations, "{:.2} {:.2} Td\n", 
                   x, self.cursor_y).unwrap();
            
            // Handle justification
            if self.alignment == TextAlign::Justified && i < lines.len() - 1 && line.len() > 1 {
                // Calculate extra space to distribute
                let spaces_count = line.iter().filter(|w| w.trim().is_empty()).count();
                if spaces_count > 0 {
                    let extra_space = content_width - line_width;
                    let space_adjustment = extra_space / spaces_count as f64;
                    
                    // Set word spacing
                    write!(&mut self.operations, "{:.2} Tw\n", space_adjustment).unwrap();
                }
            }
            
            // Show text
            self.operations.push('(');
            for ch in line_text.chars() {
                match ch {
                    '(' => self.operations.push_str("\\("),
                    ')' => self.operations.push_str("\\)"),
                    '\\' => self.operations.push_str("\\\\"),
                    '\n' => self.operations.push_str("\\n"),
                    '\r' => self.operations.push_str("\\r"),
                    '\t' => self.operations.push_str("\\t"),
                    _ => self.operations.push(ch),
                }
            }
            self.operations.push_str(") Tj\n");
            
            // Reset word spacing if it was set
            if self.alignment == TextAlign::Justified && i < lines.len() - 1 {
                self.operations.push_str("0 Tw\n");
            }
            
            // End text object
            self.operations.push_str("ET\n");
            
            // Move cursor down for next line
            self.cursor_y -= self.font_size * self.line_height;
        }
        
        Ok(self)
    }
    
    pub fn write_paragraph(&mut self, text: &str) -> Result<&mut Self> {
        self.write_wrapped(text)?;
        // Add extra space after paragraph
        self.cursor_y -= self.font_size * self.line_height * 0.5;
        Ok(self)
    }
    
    pub fn newline(&mut self) -> &mut Self {
        self.cursor_y -= self.font_size * self.line_height;
        self.cursor_x = self.margins.left;
        self
    }
    
    pub fn cursor_position(&self) -> (f64, f64) {
        (self.cursor_x, self.cursor_y)
    }
    
    pub fn generate_operations(&self) -> Vec<u8> {
        self.operations.as_bytes().to_vec()
    }
}