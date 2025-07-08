//! PDF page rotation functionality
//! 
//! This module provides functionality to rotate pages in PDF documents.

use crate::parser::{PdfReader, ParsedPage};
use crate::{Document, Page};
use super::{OperationError, OperationResult, PageRange};
use std::path::Path;
use std::io::{Read, Seek};

/// Rotation angle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationAngle {
    /// No rotation (0 degrees)
    None,
    /// 90 degrees clockwise
    Clockwise90,
    /// 180 degrees
    Rotate180,
    /// 270 degrees clockwise (90 degrees counter-clockwise)
    Clockwise270,
}

impl RotationAngle {
    /// Create from degrees
    pub fn from_degrees(degrees: i32) -> Result<Self, OperationError> {
        let normalized = degrees % 360;
        let normalized = if normalized < 0 { normalized + 360 } else { normalized };
        
        match normalized {
            0 => Ok(RotationAngle::None),
            90 => Ok(RotationAngle::Clockwise90),
            180 => Ok(RotationAngle::Rotate180),
            270 => Ok(RotationAngle::Clockwise270),
            _ => Err(OperationError::InvalidRotation(degrees)),
        }
    }
    
    /// Convert to degrees
    pub fn to_degrees(self) -> i32 {
        match self {
            RotationAngle::None => 0,
            RotationAngle::Clockwise90 => 90,
            RotationAngle::Rotate180 => 180,
            RotationAngle::Clockwise270 => 270,
        }
    }
    
    /// Combine two rotations
    pub fn combine(self, other: RotationAngle) -> RotationAngle {
        let total = (self.to_degrees() + other.to_degrees()) % 360;
        RotationAngle::from_degrees(total).unwrap()
    }
}

/// Options for page rotation
#[derive(Debug, Clone)]
pub struct RotateOptions {
    /// Pages to rotate
    pub pages: PageRange,
    /// Rotation angle
    pub angle: RotationAngle,
    /// Whether to preserve the original page size (vs adjusting for rotated content)
    pub preserve_page_size: bool,
}

impl Default for RotateOptions {
    fn default() -> Self {
        Self {
            pages: PageRange::All,
            angle: RotationAngle::Clockwise90,
            preserve_page_size: false,
        }
    }
}

/// PDF page rotator
pub struct PageRotator<R: Read + Seek> {
    reader: PdfReader<R>,
}

impl<R: Read + Seek> PageRotator<R> {
    /// Create a new page rotator
    pub fn new(reader: PdfReader<R>) -> Self {
        Self { reader }
    }
    
    /// Rotate pages according to options
    pub fn rotate(&mut self, options: &RotateOptions) -> OperationResult<Document> {
        let total_pages = self.reader.page_count()
            .map_err(|e| OperationError::ParseError(e.to_string()))? as usize;
        
        let page_indices = options.pages.get_indices(total_pages)?;
        let mut output_doc = Document::new();
        
        // Copy metadata
        if let Ok(metadata) = self.reader.metadata() {
            if let Some(title) = metadata.title {
                output_doc.set_title(&title);
            }
            if let Some(author) = metadata.author {
                output_doc.set_author(&author);
            }
            if let Some(subject) = metadata.subject {
                output_doc.set_subject(&subject);
            }
            if let Some(keywords) = metadata.keywords {
                output_doc.set_keywords(&keywords);
            }
        }
        
        // Process each page
        for page_idx in 0..total_pages {
            let parsed_page = self.reader.get_page(page_idx as u32)
                .map_err(|e| OperationError::ParseError(e.to_string()))?;
            
            // Check if this page should be rotated
            let should_rotate = page_indices.contains(&page_idx);
            
            let new_page = if should_rotate {
                self.create_rotated_page(parsed_page, options.angle, options.preserve_page_size)?
            } else {
                self.create_page_copy(parsed_page)?
            };
            
            output_doc.add_page(new_page);
        }
        
        Ok(output_doc)
    }
    
    /// Create a rotated copy of a page
    fn create_rotated_page(
        &mut self,
        parsed_page: &ParsedPage,
        angle: RotationAngle,
        preserve_size: bool,
    ) -> OperationResult<Page> {
        // Calculate the effective rotation
        let current_rotation = parsed_page.rotation;
        let new_rotation = ((current_rotation + angle.to_degrees()) % 360) as i32;
        
        // Get original dimensions
        let orig_width = parsed_page.media_box[2] - parsed_page.media_box[0];
        let orig_height = parsed_page.media_box[3] - parsed_page.media_box[1];
        
        // Calculate new dimensions based on rotation
        let (new_width, new_height) = if preserve_size {
            (orig_width, orig_height)
        } else {
            match angle {
                RotationAngle::None | RotationAngle::Rotate180 => (orig_width, orig_height),
                RotationAngle::Clockwise90 | RotationAngle::Clockwise270 => (orig_height, orig_width),
            }
        };
        
        let mut page = Page::new(new_width as f32, new_height as f32);
        
        // Get content streams
        let content_streams = parsed_page.content_streams(&mut self.reader)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;
        
        // Add rotation transformation
        match angle {
            RotationAngle::None => {
                // No rotation needed
            }
            RotationAngle::Clockwise90 => {
                // Rotate 90 degrees clockwise
                // Transform: x' = y, y' = width - x
                page.graphics()
                    .save_state()
                    .transform(0.0, 1.0, -1.0, 0.0, new_width as f32, 0.0);
            }
            RotationAngle::Rotate180 => {
                // Rotate 180 degrees
                // Transform: x' = width - x, y' = height - y
                page.graphics()
                    .save_state()
                    .transform(-1.0, 0.0, 0.0, -1.0, new_width as f32, new_height as f32);
            }
            RotationAngle::Clockwise270 => {
                // Rotate 270 degrees clockwise (90 counter-clockwise)
                // Transform: x' = height - y, y' = x
                page.graphics()
                    .save_state()
                    .transform(0.0, -1.0, 1.0, 0.0, 0.0, new_height as f32);
            }
        }
        
        // For now, add a placeholder
        // Full implementation would apply the rotation transformation to the content
        if !content_streams.is_empty() {
            page.text()
                .set_font(crate::text::Font::Helvetica, 10.0)
                .at(50.0, new_height as f32 - 50.0)
                .write(&format!("[Page rotated {} degrees - content transformation not yet implemented]", angle.to_degrees()))
                .map_err(|e| OperationError::PdfError(e))?;
        }
        
        // Restore graphics state if we transformed
        if angle != RotationAngle::None {
            page.graphics().restore_state();
        }
        
        Ok(page)
    }
    
    /// Create a copy of a page without rotation
    fn create_page_copy(&mut self, parsed_page: &ParsedPage) -> OperationResult<Page> {
        let width = parsed_page.width();
        let height = parsed_page.height();
        let mut page = Page::new(width as f32, height as f32);
        
        // Get content streams
        let content_streams = parsed_page.content_streams(&mut self.reader)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;
        
        // For now, add a placeholder
        if !content_streams.is_empty() {
            page.text()
                .set_font(crate::text::Font::Helvetica, 10.0)
                .at(50.0, height as f32 - 50.0)
                .write("[Page copied - content parsing not yet implemented]")
                .map_err(|e| OperationError::PdfError(e))?;
        }
        
        Ok(page)
    }
}

/// Rotate pages in a PDF file
pub fn rotate_pdf_pages<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    options: RotateOptions,
) -> OperationResult<()> {
    let reader = PdfReader::open(input_path)
        .map_err(|e| OperationError::ParseError(e.to_string()))?;
    
    let mut rotator = PageRotator::new(reader);
    let doc = rotator.rotate(&options)?;
    
    doc.save(output_path)?;
    Ok(())
}

/// Rotate all pages in a PDF file
pub fn rotate_all_pages<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    angle: RotationAngle,
) -> OperationResult<()> {
    let options = RotateOptions {
        pages: PageRange::All,
        angle,
        preserve_page_size: false,
    };
    
    rotate_pdf_pages(input_path, output_path, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rotation_angle() {
        assert_eq!(RotationAngle::from_degrees(0).unwrap(), RotationAngle::None);
        assert_eq!(RotationAngle::from_degrees(90).unwrap(), RotationAngle::Clockwise90);
        assert_eq!(RotationAngle::from_degrees(180).unwrap(), RotationAngle::Rotate180);
        assert_eq!(RotationAngle::from_degrees(270).unwrap(), RotationAngle::Clockwise270);
        
        // Test normalization
        assert_eq!(RotationAngle::from_degrees(360).unwrap(), RotationAngle::None);
        assert_eq!(RotationAngle::from_degrees(450).unwrap(), RotationAngle::Clockwise90);
        assert_eq!(RotationAngle::from_degrees(-90).unwrap(), RotationAngle::Clockwise270);
        
        // Test invalid angles
        assert!(RotationAngle::from_degrees(45).is_err());
        assert!(RotationAngle::from_degrees(135).is_err());
    }
    
    #[test]
    fn test_rotation_combine() {
        let r1 = RotationAngle::Clockwise90;
        let r2 = RotationAngle::Clockwise90;
        assert_eq!(r1.combine(r2), RotationAngle::Rotate180);
        
        let r3 = RotationAngle::Clockwise270;
        let r4 = RotationAngle::Clockwise90;
        assert_eq!(r3.combine(r4), RotationAngle::None);
    }
}