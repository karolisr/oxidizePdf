//! PDF merging functionality
//! 
//! This module provides functionality to merge multiple PDF documents into a single file.

use crate::parser::{PdfReader, ParsedPage};
use crate::{Document, Page};
use super::{OperationError, OperationResult, PageRange};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::collections::HashMap;

/// Options for PDF merging
#[derive(Debug, Clone)]
pub struct MergeOptions {
    /// Page ranges to include from each input file
    pub page_ranges: Option<Vec<PageRange>>,
    /// Whether to preserve bookmarks/outlines
    pub preserve_bookmarks: bool,
    /// Whether to preserve form fields
    pub preserve_forms: bool,
    /// Whether to optimize the output
    pub optimize: bool,
    /// How to handle metadata
    pub metadata_mode: MetadataMode,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            page_ranges: None,
            preserve_bookmarks: true,
            preserve_forms: false,
            optimize: false,
            metadata_mode: MetadataMode::FromFirst,
        }
    }
}

/// How to handle metadata when merging
#[derive(Debug, Clone)]
pub enum MetadataMode {
    /// Use metadata from the first document
    FromFirst,
    /// Use metadata from a specific document (by index)
    FromDocument(usize),
    /// Use custom metadata
    Custom {
        title: Option<String>,
        author: Option<String>,
        subject: Option<String>,
        keywords: Option<String>,
    },
    /// Don't set any metadata
    None,
}

/// Input specification for merging
#[derive(Debug)]
pub struct MergeInput {
    /// Path to the PDF file
    pub path: PathBuf,
    /// Optional page range to include
    pub pages: Option<PageRange>,
}

impl MergeInput {
    /// Create a new merge input that includes all pages
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            pages: None,
        }
    }
    
    /// Create a merge input with specific pages
    pub fn with_pages<P: Into<PathBuf>>(path: P, pages: PageRange) -> Self {
        Self {
            path: path.into(),
            pages: Some(pages),
        }
    }
}

/// PDF merger
pub struct PdfMerger {
    inputs: Vec<MergeInput>,
    options: MergeOptions,
    /// Object number mapping for each input document
    object_mappings: Vec<HashMap<u32, u32>>,
    /// Next available object number
    next_object_num: u32,
}

impl PdfMerger {
    /// Create a new PDF merger
    pub fn new(options: MergeOptions) -> Self {
        Self {
            inputs: Vec::new(),
            options,
            object_mappings: Vec::new(),
            next_object_num: 1,
        }
    }
    
    /// Add an input file to merge
    pub fn add_input(&mut self, input: MergeInput) {
        self.inputs.push(input);
    }
    
    /// Add multiple input files
    pub fn add_inputs(&mut self, inputs: impl IntoIterator<Item = MergeInput>) {
        self.inputs.extend(inputs);
    }
    
    /// Merge all input files into a single document
    pub fn merge(&mut self) -> OperationResult<Document> {
        if self.inputs.is_empty() {
            return Err(OperationError::NoPagesToProcess);
        }
        
        let mut output_doc = Document::new();
        
        // Process each input file
        for (input_idx, input) in self.inputs.iter().enumerate() {
            let mut reader = PdfReader::open(&input.path)
                .map_err(|e| OperationError::ParseError(
                    format!("Failed to open {}: {}", input.path.display(), e)
                ))?;
            
            // Initialize object mapping for this document
            self.object_mappings.push(HashMap::new());
            
            // Get page range
            let total_pages = reader.page_count()
                .map_err(|e| OperationError::ParseError(e.to_string()))? as usize;
            
            let page_range = input.pages.as_ref()
                .unwrap_or(&PageRange::All);
            
            let page_indices = page_range.get_indices(total_pages)?;
            
            // Extract and add pages
            for page_idx in page_indices {
                let parsed_page = reader.get_page(page_idx as u32)
                    .map_err(|e| OperationError::ParseError(e.to_string()))?;
                
                let new_page = self.convert_page_for_merge(parsed_page, &mut reader, input_idx)?;
                output_doc.add_page(new_page);
            }
            
            // Handle metadata for the first document or specified document
            match &self.options.metadata_mode {
                MetadataMode::FromFirst if input_idx == 0 => {
                    self.copy_metadata(&mut reader, &mut output_doc)?;
                }
                MetadataMode::FromDocument(idx) if input_idx == *idx => {
                    self.copy_metadata(&mut reader, &mut output_doc)?;
                }
                _ => {}
            }
        }
        
        // Apply custom metadata if specified
        if let MetadataMode::Custom { title, author, subject, keywords } = &self.options.metadata_mode {
            if let Some(title) = title {
                output_doc.set_title(title);
            }
            if let Some(author) = author {
                output_doc.set_author(author);
            }
            if let Some(subject) = subject {
                output_doc.set_subject(subject);
            }
            if let Some(keywords) = keywords {
                output_doc.set_keywords(keywords);
            }
        }
        
        Ok(output_doc)
    }
    
    /// Merge files and save to output path
    pub fn merge_to_file<P: AsRef<Path>>(&mut self, output_path: P) -> OperationResult<()> {
        let doc = self.merge()?;
        doc.save(output_path)?;
        Ok(())
    }
    
    /// Convert a page for merging, handling object renumbering
    fn convert_page_for_merge(
        &mut self,
        parsed_page: &ParsedPage,
        reader: &mut PdfReader<File>,
        input_idx: usize,
    ) -> OperationResult<Page> {
        // Create new page with same dimensions
        let width = parsed_page.width();
        let height = parsed_page.height();
        let mut page = Page::new(width as f32, height as f32);
        
        // Get content streams
        let content_streams = parsed_page.content_streams(reader)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;
        
        // For now, add a placeholder
        // Full implementation would require parsing and renumbering all object references
        if !content_streams.is_empty() {
            page.text()
                .set_font(crate::text::Font::Helvetica, 10.0)
                .at(50.0, height as f32 - 50.0)
                .write(&format!("[Page from document {} - content parsing not yet implemented]", input_idx + 1))
                .map_err(|e| OperationError::PdfError(e))?;
        }
        
        Ok(page)
    }
    
    /// Copy metadata from source to destination document
    fn copy_metadata(
        &self,
        reader: &mut PdfReader<File>,
        doc: &mut Document,
    ) -> OperationResult<()> {
        if let Ok(metadata) = reader.metadata() {
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
        Ok(())
    }
    
    /// Get the next available object number and increment
    fn allocate_object_number(&mut self) -> u32 {
        let num = self.next_object_num;
        self.next_object_num += 1;
        num
    }
    
    /// Map an object number from an input document to the merged document
    fn map_object_number(&mut self, input_idx: usize, old_num: u32) -> u32 {
        let mapping = &mut self.object_mappings[input_idx];
        
        if let Some(&new_num) = mapping.get(&old_num) {
            new_num
        } else {
            let new_num = self.allocate_object_number();
            mapping.insert(old_num, new_num);
            new_num
        }
    }
}

/// Merge multiple PDF files into one
pub fn merge_pdfs<P: AsRef<Path>>(
    inputs: Vec<MergeInput>,
    output_path: P,
    options: MergeOptions,
) -> OperationResult<()> {
    let mut merger = PdfMerger::new(options);
    merger.add_inputs(inputs);
    merger.merge_to_file(output_path)
}

/// Simple merge of multiple PDF files with default options
pub fn merge_pdf_files<P: AsRef<Path>, Q: AsRef<Path>>(
    input_paths: &[P],
    output_path: Q,
) -> OperationResult<()> {
    let inputs: Vec<MergeInput> = input_paths.iter()
        .map(|p| MergeInput::new(p.as_ref()))
        .collect();
    
    merge_pdfs(inputs, output_path, MergeOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merge_options_default() {
        let options = MergeOptions::default();
        assert!(options.page_ranges.is_none());
        assert!(options.preserve_bookmarks);
        assert!(!options.preserve_forms);
        assert!(!options.optimize);
        assert!(matches!(options.metadata_mode, MetadataMode::FromFirst));
    }
    
    #[test]
    fn test_merge_input_creation() {
        let input = MergeInput::new("test.pdf");
        assert_eq!(input.path, PathBuf::from("test.pdf"));
        assert!(input.pages.is_none());
        
        let input_with_pages = MergeInput::with_pages(
            "test.pdf",
            PageRange::Range(0, 4)
        );
        assert!(input_with_pages.pages.is_some());
    }
}