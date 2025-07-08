//! PDF splitting functionality
//! 
//! This module provides functionality to split PDF documents into multiple files
//! based on page ranges or other criteria.

use crate::parser::{PdfReader, ParsedPage};
use crate::{Document, Page};
use crate::writer::PdfWriter;
use super::{OperationError, OperationResult, PageRange};
use std::path::{Path, PathBuf};
use std::io::{Read, Seek};

/// Options for PDF splitting
#[derive(Debug, Clone)]
pub struct SplitOptions {
    /// How to split the document
    pub mode: SplitMode,
    /// Output file naming pattern
    pub output_pattern: String,
    /// Whether to preserve document metadata
    pub preserve_metadata: bool,
    /// Whether to optimize output files
    pub optimize: bool,
}

impl Default for SplitOptions {
    fn default() -> Self {
        Self {
            mode: SplitMode::SinglePages,
            output_pattern: "page_{}.pdf".to_string(),
            preserve_metadata: true,
            optimize: false,
        }
    }
}

/// Split mode specification
#[derive(Debug, Clone)]
pub enum SplitMode {
    /// Split into single pages
    SinglePages,
    /// Split by page ranges
    Ranges(Vec<PageRange>),
    /// Split into chunks of N pages
    ChunkSize(usize),
    /// Split at specific page numbers (creates files before each split point)
    SplitAt(Vec<usize>),
}

/// PDF splitter
pub struct PdfSplitter<R: Read + Seek> {
    reader: PdfReader<R>,
    options: SplitOptions,
}

impl<R: Read + Seek> PdfSplitter<R> {
    /// Create a new PDF splitter
    pub fn new(reader: PdfReader<R>, options: SplitOptions) -> Self {
        Self { reader, options }
    }
    
    /// Split the PDF according to the options
    pub fn split(&mut self) -> OperationResult<Vec<PathBuf>> {
        let total_pages = self.reader.page_count()
            .map_err(|e| OperationError::ParseError(e.to_string()))? as usize;
        
        if total_pages == 0 {
            return Err(OperationError::NoPagesToProcess);
        }
        
        let ranges = match &self.options.mode {
            SplitMode::SinglePages => {
                // Create a range for each page
                (0..total_pages).map(PageRange::Single).collect()
            }
            SplitMode::Ranges(ranges) => ranges.clone(),
            SplitMode::ChunkSize(size) => {
                // Create ranges for chunks
                let mut ranges = Vec::new();
                let mut start = 0;
                while start < total_pages {
                    let end = (start + size - 1).min(total_pages - 1);
                    ranges.push(PageRange::Range(start, end));
                    start += size;
                }
                ranges
            }
            SplitMode::SplitAt(split_points) => {
                // Create ranges between split points
                let mut ranges = Vec::new();
                let mut start = 0;
                
                for &split_point in split_points {
                    if split_point > 0 && split_point < total_pages {
                        ranges.push(PageRange::Range(start, split_point - 1));
                        start = split_point;
                    }
                }
                
                // Add the last range
                if start < total_pages {
                    ranges.push(PageRange::Range(start, total_pages - 1));
                }
                
                ranges
            }
        };
        
        // Process each range
        let mut output_files = Vec::new();
        
        for (index, range) in ranges.iter().enumerate() {
            let output_path = self.format_output_path(index, &range);
            self.extract_range(range, &output_path)?;
            output_files.push(output_path);
        }
        
        Ok(output_files)
    }
    
    /// Extract a page range to a new PDF file
    fn extract_range(&mut self, range: &PageRange, output_path: &Path) -> OperationResult<()> {
        let total_pages = self.reader.page_count()
            .map_err(|e| OperationError::ParseError(e.to_string()))? as usize;
        
        let indices = range.get_indices(total_pages)?;
        if indices.is_empty() {
            return Err(OperationError::NoPagesToProcess);
        }
        
        // Create new document
        let mut doc = Document::new();
        
        // Copy metadata if requested
        if self.options.preserve_metadata {
            if let Ok(metadata) = self.reader.metadata() {
                if let Some(title) = metadata.title {
                    doc.set_title(&title);
                }
                if let Some(author) = metadata.author {
                    doc.set_author(&author);
                }
                if let Some(subject) = metadata.subject {
                    doc.set_subject(&subject);
                }
                if let Some(keywords) = metadata.keywords {
                    doc.set_keywords(&keywords);
                }
            }
        }
        
        // Extract and add pages
        for &page_idx in &indices {
            let parsed_page = self.reader.get_page(page_idx as u32)
                .map_err(|e| OperationError::ParseError(e.to_string()))?;
            
            // Convert parsed page to new page
            let new_page = self.convert_page(parsed_page)?;
            doc.add_page(new_page);
        }
        
        // Save the document
        doc.save(output_path)?;
        
        Ok(())
    }
    
    /// Convert a parsed page to a new page
    fn convert_page(&mut self, parsed_page: &ParsedPage) -> OperationResult<Page> {
        // Create new page with same dimensions
        let width = parsed_page.width();
        let height = parsed_page.height();
        let mut page = Page::new(width as f32, height as f32);
        
        // Set rotation if needed
        if parsed_page.rotation != 0 {
            // TODO: Implement rotation in Page
            // For now, we'll handle this when we implement the rotation feature
        }
        
        // Get content streams
        let content_streams = parsed_page.content_streams(&mut self.reader)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;
        
        // For now, we'll create a placeholder that copies the content
        // In a full implementation, we would parse and recreate the content
        // This is a limitation that we'll address when implementing content stream parsing
        
        if !content_streams.is_empty() {
            // Add a note about the limitation
            page.text()
                .set_font(crate::text::Font::Helvetica, 10.0)
                .at(50.0, height as f32 - 50.0)
                .write("[Page extracted - content parsing not yet implemented]")
                .map_err(|e| OperationError::PdfError(e))?;
        }
        
        Ok(page)
    }
    
    /// Format the output path based on the pattern
    fn format_output_path(&self, index: usize, range: &PageRange) -> PathBuf {
        let filename = match range {
            PageRange::Single(page) => {
                self.options.output_pattern
                    .replace("{}", &(page + 1).to_string())
                    .replace("{n}", &(index + 1).to_string())
                    .replace("{page}", &(page + 1).to_string())
            }
            PageRange::Range(start, end) => {
                self.options.output_pattern
                    .replace("{}", &format!("{}-{}", start + 1, end + 1))
                    .replace("{n}", &(index + 1).to_string())
                    .replace("{start}", &(start + 1).to_string())
                    .replace("{end}", &(end + 1).to_string())
            }
            _ => {
                self.options.output_pattern
                    .replace("{}", &(index + 1).to_string())
                    .replace("{n}", &(index + 1).to_string())
            }
        };
        
        PathBuf::from(filename)
    }
}

/// Split a PDF file by page ranges
pub fn split_pdf<P: AsRef<Path>>(
    input_path: P,
    options: SplitOptions,
) -> OperationResult<Vec<PathBuf>> {
    let reader = PdfReader::open(input_path)
        .map_err(|e| OperationError::ParseError(e.to_string()))?;
    
    let mut splitter = PdfSplitter::new(reader, options);
    splitter.split()
}

/// Split a PDF file into single pages
pub fn split_into_pages<P: AsRef<Path>>(
    input_path: P,
    output_pattern: &str,
) -> OperationResult<Vec<PathBuf>> {
    let options = SplitOptions {
        mode: SplitMode::SinglePages,
        output_pattern: output_pattern.to_string(),
        ..Default::default()
    };
    
    split_pdf(input_path, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_split_options_default() {
        let options = SplitOptions::default();
        assert!(matches!(options.mode, SplitMode::SinglePages));
        assert_eq!(options.output_pattern, "page_{}.pdf");
        assert!(options.preserve_metadata);
        assert!(!options.optimize);
    }
    
    #[test]
    fn test_format_output_path() {
        let options = SplitOptions {
            output_pattern: "output_page_{}.pdf".to_string(),
            ..Default::default()
        };
        
        let reader = PdfReader::open("test.pdf");
        // Note: This test would need a valid PDF file to work properly
        // For now, we're just testing the logic
    }
}