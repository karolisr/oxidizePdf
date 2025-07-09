//! PDF Document wrapper with improved architecture
//! 
//! This module provides a higher-level interface for PDF parsing that solves
//! the borrow checker issues by using interior mutability and separation of concerns.

use super::{ParseError, ParseResult};
use super::reader::PdfReader;
use super::page_tree::{PageTree, ParsedPage};
use super::objects::{PdfObject, PdfDictionary};
use std::io::{Read, Seek};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

/// Resource manager for caching PDF objects
pub struct ResourceManager {
    /// Cached objects by (obj_num, gen_num)
    object_cache: RefCell<HashMap<(u32, u16), PdfObject>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            object_cache: RefCell::new(HashMap::new()),
        }
    }
    
    /// Get an object from cache
    pub fn get_cached(&self, obj_ref: (u32, u16)) -> Option<PdfObject> {
        self.object_cache.borrow().get(&obj_ref).cloned()
    }
    
    /// Cache an object
    pub fn cache_object(&self, obj_ref: (u32, u16), obj: PdfObject) {
        self.object_cache.borrow_mut().insert(obj_ref, obj);
    }
    
    /// Clear the cache
    pub fn clear_cache(&self) {
        self.object_cache.borrow_mut().clear();
    }
}

/// High-level PDF document interface
pub struct PdfDocument<R: Read + Seek> {
    /// The underlying PDF reader
    reader: RefCell<PdfReader<R>>,
    /// Page tree navigator
    page_tree: RefCell<Option<PageTree>>,
    /// Resource manager for object caching
    resources: Rc<ResourceManager>,
    /// Document metadata cache
    metadata_cache: RefCell<Option<super::reader::DocumentMetadata>>,
}

impl<R: Read + Seek> PdfDocument<R> {
    /// Create a new PDF document from a reader
    pub fn new(reader: PdfReader<R>) -> Self {
        Self {
            reader: RefCell::new(reader),
            page_tree: RefCell::new(None),
            resources: Rc::new(ResourceManager::new()),
            metadata_cache: RefCell::new(None),
        }
    }
    
    /// Get the PDF version
    pub fn version(&self) -> ParseResult<String> {
        Ok(self.reader.borrow().version().to_string())
    }
    
    /// Get the number of pages
    pub fn page_count(&self) -> ParseResult<u32> {
        self.reader.borrow_mut().page_count()
    }
    
    /// Get document metadata
    pub fn metadata(&self) -> ParseResult<super::reader::DocumentMetadata> {
        // Check cache first
        if let Some(metadata) = self.metadata_cache.borrow().as_ref() {
            return Ok(metadata.clone());
        }
        
        // Load metadata
        let metadata = self.reader.borrow_mut().metadata()?;
        self.metadata_cache.borrow_mut().replace(metadata.clone());
        Ok(metadata)
    }
    
    /// Initialize the page tree if not already done
    fn ensure_page_tree(&self) -> ParseResult<()> {
        if self.page_tree.borrow().is_none() {
            let page_count = self.page_count()?;
            let pages_dict = self.load_pages_dict()?;
            let page_tree = PageTree::new_with_pages_dict(page_count, pages_dict);
            self.page_tree.borrow_mut().replace(page_tree);
        }
        Ok(())
    }
    
    /// Load the pages dictionary
    fn load_pages_dict(&self) -> ParseResult<PdfDictionary> {
        let mut reader = self.reader.borrow_mut();
        let pages = reader.pages()?;
        Ok(pages.clone())
    }
    
    /// Get a page by index (0-based)
    pub fn get_page(&self, index: u32) -> ParseResult<ParsedPage> {
        self.ensure_page_tree()?;
        
        // First check if page is already loaded
        if let Some(page_tree) = self.page_tree.borrow().as_ref() {
            if let Some(page) = page_tree.get_cached_page(index) {
                return Ok(page.clone());
            }
        }
        
        // Load the page
        let page = self.load_page_at_index(index)?;
        
        // Cache it
        if let Some(page_tree) = self.page_tree.borrow_mut().as_mut() {
            page_tree.cache_page(index, page.clone());
        }
        
        Ok(page)
    }
    
    /// Load a specific page by index
    fn load_page_at_index(&self, index: u32) -> ParseResult<ParsedPage> {
        // Get the pages root
        let pages_dict = self.load_pages_dict()?;
        
        // Navigate to the specific page
        let page_info = self.find_page_in_tree(&pages_dict, index, 0, None)?;
        
        Ok(page_info)
    }
    
    /// Find a page in the page tree
    fn find_page_in_tree(
        &self,
        node: &PdfDictionary,
        target_index: u32,
        current_index: u32,
        inherited: Option<&PdfDictionary>,
    ) -> ParseResult<ParsedPage> {
        let node_type = node.get_type()
            .ok_or_else(|| ParseError::MissingKey("Type".to_string()))?;
        
        match node_type {
            "Pages" => {
                // This is a page tree node
                let kids = node.get("Kids")
                    .and_then(|obj| obj.as_array())
                    .ok_or_else(|| ParseError::MissingKey("Kids".to_string()))?;
                
                // Merge inherited attributes
                let mut merged_inherited = inherited.cloned().unwrap_or_else(PdfDictionary::new);
                
                // Inheritable attributes
                for key in ["Resources", "MediaBox", "CropBox", "Rotate"] {
                    if let Some(value) = node.get(key) {
                        if !merged_inherited.contains_key(key) {
                            merged_inherited.insert(key.to_string(), value.clone());
                        }
                    }
                }
                
                // Find which kid contains our target page
                let mut current_idx = current_index;
                for kid_ref in &kids.0 {
                    let kid_ref = kid_ref.as_reference()
                        .ok_or_else(|| ParseError::SyntaxError {
                            position: 0,
                            message: "Kids array must contain references".to_string(),
                        })?;
                    
                    // Get the kid object
                    let kid_obj = self.get_object(kid_ref.0, kid_ref.1)?;
                    let kid_dict = kid_obj.as_dict()
                        .ok_or_else(|| ParseError::SyntaxError {
                            position: 0,
                            message: "Page tree node must be a dictionary".to_string(),
                        })?;
                    
                    let kid_type = kid_dict.get_type()
                        .ok_or_else(|| ParseError::MissingKey("Type".to_string()))?;
                    
                    let count = if kid_type == "Pages" {
                        kid_dict.get("Count")
                            .and_then(|obj| obj.as_integer())
                            .ok_or_else(|| ParseError::MissingKey("Count".to_string()))? as u32
                    } else {
                        1
                    };
                    
                    if target_index < current_idx + count {
                        // Found the right subtree/page
                        if kid_type == "Page" {
                            // This is the page we want
                            return self.create_parsed_page(kid_ref, kid_dict, Some(&merged_inherited));
                        } else {
                            // Recurse into this subtree
                            return self.find_page_in_tree(
                                kid_dict,
                                target_index,
                                current_idx,
                                Some(&merged_inherited),
                            );
                        }
                    }
                    
                    current_idx += count;
                }
                
                Err(ParseError::SyntaxError {
                    position: 0,
                    message: "Page not found in tree".to_string(),
                })
            }
            "Page" => {
                // This is a page object
                if target_index != current_index {
                    return Err(ParseError::SyntaxError {
                        position: 0,
                        message: "Page index mismatch".to_string(),
                    });
                }
                
                // We need the reference, but we don't have it here
                // This case shouldn't happen if we're navigating properly
                Err(ParseError::SyntaxError {
                    position: 0,
                    message: "Direct page object without reference".to_string(),
                })
            }
            _ => Err(ParseError::SyntaxError {
                position: 0,
                message: format!("Invalid page tree node type: {}", node_type),
            }),
        }
    }
    
    /// Create a ParsedPage from a page dictionary
    fn create_parsed_page(
        &self,
        obj_ref: (u32, u16),
        page_dict: &PdfDictionary,
        inherited: Option<&PdfDictionary>,
    ) -> ParseResult<ParsedPage> {
        // Extract page attributes
        let media_box = self.get_rectangle(page_dict, inherited, "MediaBox")?
            .ok_or_else(|| ParseError::MissingKey("MediaBox".to_string()))?;
        
        let crop_box = self.get_rectangle(page_dict, inherited, "CropBox")?;
        
        let rotation = self.get_integer(page_dict, inherited, "Rotate")?
            .unwrap_or(0) as i32;
        
        // Get inherited resources
        let inherited_resources = if let Some(inherited) = inherited {
            inherited.get("Resources").and_then(|r| r.as_dict()).cloned()
        } else {
            None
        };
        
        Ok(ParsedPage {
            obj_ref,
            dict: page_dict.clone(),
            inherited_resources,
            media_box,
            crop_box,
            rotation,
        })
    }
    
    /// Get a rectangle value
    fn get_rectangle(
        &self,
        node: &PdfDictionary,
        inherited: Option<&PdfDictionary>,
        key: &str,
    ) -> ParseResult<Option<[f64; 4]>> {
        let array = node.get(key)
            .or_else(|| inherited.and_then(|i| i.get(key)));
        
        if let Some(array) = array.and_then(|obj| obj.as_array()) {
            if array.len() != 4 {
                return Err(ParseError::SyntaxError {
                    position: 0,
                    message: format!("{} must have 4 elements", key),
                });
            }
            
            let rect = [
                array.get(0).unwrap().as_real().unwrap_or(0.0),
                array.get(1).unwrap().as_real().unwrap_or(0.0),
                array.get(2).unwrap().as_real().unwrap_or(0.0),
                array.get(3).unwrap().as_real().unwrap_or(0.0),
            ];
            
            Ok(Some(rect))
        } else {
            Ok(None)
        }
    }
    
    /// Get an integer value
    fn get_integer(
        &self,
        node: &PdfDictionary,
        inherited: Option<&PdfDictionary>,
        key: &str,
    ) -> ParseResult<Option<i64>> {
        let value = node.get(key)
            .or_else(|| inherited.and_then(|i| i.get(key)));
        
        Ok(value.and_then(|obj| obj.as_integer()))
    }
    
    /// Get an object by reference
    pub fn get_object(&self, obj_num: u32, gen_num: u16) -> ParseResult<PdfObject> {
        // Check resource cache first
        if let Some(obj) = self.resources.get_cached((obj_num, gen_num)) {
            return Ok(obj);
        }
        
        // Load from reader
        let obj = {
            let mut reader = self.reader.borrow_mut();
            reader.get_object(obj_num, gen_num)?.clone()
        };
        
        // Cache it
        self.resources.cache_object((obj_num, gen_num), obj.clone());
        
        Ok(obj)
    }
    
    /// Resolve a reference
    pub fn resolve(&self, obj: &PdfObject) -> ParseResult<PdfObject> {
        match obj {
            PdfObject::Reference(obj_num, gen_num) => {
                self.get_object(*obj_num, *gen_num)
            }
            _ => Ok(obj.clone()),
        }
    }
    
    /// Get content streams for a page
    pub fn get_page_content_streams(&self, page: &ParsedPage) -> ParseResult<Vec<Vec<u8>>> {
        let mut streams = Vec::new();
        
        if let Some(contents) = page.dict.get("Contents") {
            let resolved_contents = self.resolve(contents)?;
            
            match &resolved_contents {
                PdfObject::Stream(stream) => {
                    streams.push(stream.decode()?);
                }
                PdfObject::Array(array) => {
                    for item in &array.0 {
                        let resolved = self.resolve(item)?;
                        if let PdfObject::Stream(stream) = resolved {
                            streams.push(stream.decode()?);
                        }
                    }
                }
                _ => return Err(ParseError::SyntaxError {
                    position: 0,
                    message: "Contents must be a stream or array of streams".to_string(),
                }),
            }
        }
        
        Ok(streams)
    }
    
    /// Extract text from all pages
    pub fn extract_text(&self) -> ParseResult<Vec<crate::text::ExtractedText>> {
        let extractor = crate::text::TextExtractor::new();
        extractor.extract_from_document(self)
    }
    
    /// Extract text from a specific page
    pub fn extract_text_from_page(&self, page_index: u32) -> ParseResult<crate::text::ExtractedText> {
        let extractor = crate::text::TextExtractor::new();
        extractor.extract_from_page(self, page_index)
    }
    
    /// Extract text with custom options
    pub fn extract_text_with_options(&self, options: crate::text::ExtractionOptions) -> ParseResult<Vec<crate::text::ExtractedText>> {
        let extractor = crate::text::TextExtractor::with_options(options);
        extractor.extract_from_document(self)
    }
}