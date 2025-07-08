//! PDF Page Tree Parser
//! 
//! Handles navigation and extraction of pages from the PDF page tree structure

use super::{ParseError, ParseResult};
use super::objects::{PdfObject, PdfDictionary, PdfArray, PdfStream};
use super::reader::PdfReader;
use std::io::{Read, Seek};
use std::collections::HashMap;

/// Represents a single page in the PDF
#[derive(Debug, Clone)]
pub struct ParsedPage {
    /// Object reference to this page
    pub obj_ref: (u32, u16),
    /// Page dictionary
    pub dict: PdfDictionary,
    /// Inherited resources (merged from parent nodes)
    pub inherited_resources: Option<PdfDictionary>,
    /// MediaBox (page dimensions)
    pub media_box: [f64; 4],
    /// CropBox (visible area)
    pub crop_box: Option<[f64; 4]>,
    /// Page rotation in degrees
    pub rotation: i32,
}

/// Page tree navigator
pub struct PageTree {
    /// Total number of pages
    page_count: u32,
    /// Cached pages by index
    pages: HashMap<u32, ParsedPage>,
}

impl PageTree {
    /// Create a new page tree navigator
    pub fn new(page_count: u32) -> Self {
        Self {
            page_count,
            pages: HashMap::new(),
        }
    }
    
    /// Get a page by index (0-based)
    pub fn get_page<R: Read + Seek>(
        &mut self,
        reader: &mut PdfReader<R>,
        index: u32,
    ) -> ParseResult<&ParsedPage> {
        if index >= self.page_count {
            return Err(ParseError::SyntaxError {
                position: 0,
                message: format!("Page index {} out of bounds (total: {})", index, self.page_count),
            });
        }
        
        // Check cache
        if self.pages.contains_key(&index) {
            return Ok(&self.pages[&index]);
        }
        
        // Load page
        let pages_root = reader.pages()?;
        let page = self.load_page_at_index(reader, pages_root, index, None)?;
        self.pages.insert(index, page);
        
        Ok(&self.pages[&index])
    }
    
    /// Load a specific page by traversing the page tree
    fn load_page_at_index<R: Read + Seek>(
        &self,
        reader: &mut PdfReader<R>,
        node: &PdfDictionary,
        target_index: u32,
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
                
                // Inheritable attributes: Resources, MediaBox, CropBox, Rotate
                if let Some(resources) = node.get("Resources") {
                    if !merged_inherited.contains_key("Resources") {
                        merged_inherited.insert("Resources".to_string(), resources.clone());
                    }
                }
                if let Some(media_box) = node.get("MediaBox") {
                    if !merged_inherited.contains_key("MediaBox") {
                        merged_inherited.insert("MediaBox".to_string(), media_box.clone());
                    }
                }
                if let Some(crop_box) = node.get("CropBox") {
                    if !merged_inherited.contains_key("CropBox") {
                        merged_inherited.insert("CropBox".to_string(), crop_box.clone());
                    }
                }
                if let Some(rotate) = node.get("Rotate") {
                    if !merged_inherited.contains_key("Rotate") {
                        merged_inherited.insert("Rotate".to_string(), rotate.clone());
                    }
                }
                
                // Find which kid contains our target page
                let mut current_index = 0;
                for kid_ref in &kids.0 {
                    let kid_ref = kid_ref.as_reference()
                        .ok_or_else(|| ParseError::SyntaxError {
                            position: 0,
                            message: "Kids array must contain references".to_string(),
                        })?;
                    
                    let kid_obj = reader.get_object(kid_ref.0, kid_ref.1)?;
                    let kid_dict = kid_obj.as_dict()
                        .ok_or_else(|| ParseError::SyntaxError {
                            position: 0,
                            message: "Page tree node must be a dictionary".to_string(),
                        })?;
                    
                    let kid_type = kid_dict.get_type()
                        .ok_or_else(|| ParseError::MissingKey("Type".to_string()))?;
                    
                    let count = if kid_type == "Pages" {
                        // This is another page tree node
                        kid_dict.get("Count")
                            .and_then(|obj| obj.as_integer())
                            .ok_or_else(|| ParseError::MissingKey("Count".to_string()))? as u32
                    } else {
                        // This is a page
                        1
                    };
                    
                    if target_index < current_index + count {
                        // Found the right subtree/page
                        return self.load_page_at_index(
                            reader,
                            kid_dict,
                            target_index - current_index,
                            Some(&merged_inherited),
                        );
                    }
                    
                    current_index += count;
                }
                
                Err(ParseError::SyntaxError {
                    position: 0,
                    message: "Page not found in tree".to_string(),
                })
            }
            "Page" => {
                // This is a page object
                if target_index != 0 {
                    return Err(ParseError::SyntaxError {
                        position: 0,
                        message: "Page index mismatch".to_string(),
                    });
                }
                
                // Get page reference (we need to track back from the current node)
                // For now, use a placeholder
                let obj_ref = (0, 0); // TODO: Get actual reference
                
                // Extract page attributes
                let media_box = Self::get_rectangle(node, inherited, "MediaBox")?
                    .ok_or_else(|| ParseError::MissingKey("MediaBox".to_string()))?;
                
                let crop_box = Self::get_rectangle(node, inherited, "CropBox")?;
                
                let rotation = Self::get_integer(node, inherited, "Rotate")?.unwrap_or(0) as i32;
                
                // Get resources
                let inherited_resources = if let Some(inherited) = inherited {
                    inherited.get("Resources").and_then(|r| r.as_dict()).cloned()
                } else {
                    None
                };
                
                Ok(ParsedPage {
                    obj_ref,
                    dict: node.clone(),
                    inherited_resources,
                    media_box,
                    crop_box,
                    rotation,
                })
            }
            _ => Err(ParseError::SyntaxError {
                position: 0,
                message: format!("Invalid page tree node type: {}", node_type),
            }),
        }
    }
    
    /// Get a rectangle value, checking both node and inherited dictionaries
    fn get_rectangle(
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
    
    /// Get an integer value, checking both node and inherited dictionaries
    fn get_integer(
        node: &PdfDictionary,
        inherited: Option<&PdfDictionary>,
        key: &str,
    ) -> ParseResult<Option<i64>> {
        let value = node.get(key)
            .or_else(|| inherited.and_then(|i| i.get(key)));
        
        Ok(value.and_then(|obj| obj.as_integer()))
    }
}

impl ParsedPage {
    /// Get the page width
    pub fn width(&self) -> f64 {
        match self.rotation {
            90 | 270 => self.media_box[3] - self.media_box[1],
            _ => self.media_box[2] - self.media_box[0],
        }
    }
    
    /// Get the page height
    pub fn height(&self) -> f64 {
        match self.rotation {
            90 | 270 => self.media_box[2] - self.media_box[0],
            _ => self.media_box[3] - self.media_box[1],
        }
    }
    
    /// Get the content streams for this page
    pub fn content_streams<R: Read + Seek>(
        &self,
        reader: &mut PdfReader<R>,
    ) -> ParseResult<Vec<Vec<u8>>> {
        let mut streams = Vec::new();
        
        if let Some(contents) = self.dict.get("Contents") {
            match reader.resolve(contents)? {
                PdfObject::Stream(stream) => {
                    streams.push(stream.decode()?);
                }
                PdfObject::Array(array) => {
                    for obj in &array.0 {
                        if let Some(stream) = reader.resolve(obj)?.as_stream() {
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
    
    /// Get the effective resources for this page (including inherited)
    pub fn get_resources(&self) -> Option<&PdfDictionary> {
        self.dict.get("Resources")
            .and_then(|r| r.as_dict())
            .or(self.inherited_resources.as_ref())
    }
    
    /// Clone this page with all its resources
    pub fn clone_with_resources(&self) -> Self {
        let mut cloned = self.clone();
        
        // Merge inherited resources into the page dictionary if needed
        if let Some(inherited) = &self.inherited_resources {
            if !cloned.dict.contains_key("Resources") {
                cloned.dict.insert("Resources".to_string(), 
                    PdfObject::Dictionary(inherited.clone()));
            }
        }
        
        cloned
    }
    
    /// Get all referenced objects from this page (for extraction)
    pub fn get_referenced_objects<R: Read + Seek>(
        &self,
        reader: &mut PdfReader<R>,
    ) -> ParseResult<HashMap<(u32, u16), PdfObject>> {
        let mut objects = HashMap::new();
        let mut to_process = Vec::new();
        
        // Start with Contents
        if let Some(contents) = self.dict.get("Contents") {
            Self::collect_references(contents, &mut to_process);
        }
        
        // Add Resources
        if let Some(resources) = self.get_resources() {
            for (_, value) in &resources.0 {
                Self::collect_references(value, &mut to_process);
            }
        }
        
        // Process all references
        while let Some((obj_num, gen_num)) = to_process.pop() {
            if !objects.contains_key(&(obj_num, gen_num)) {
                let obj = reader.get_object(obj_num, gen_num)?;
                
                // Collect nested references
                Self::collect_references_from_object(obj, &mut to_process);
                
                objects.insert((obj_num, gen_num), obj.clone());
            }
        }
        
        Ok(objects)
    }
    
    /// Collect object references from a PDF object
    fn collect_references(obj: &PdfObject, refs: &mut Vec<(u32, u16)>) {
        match obj {
            PdfObject::Reference(obj_num, gen_num) => {
                refs.push((*obj_num, *gen_num));
            }
            PdfObject::Array(array) => {
                for item in &array.0 {
                    Self::collect_references(item, refs);
                }
            }
            PdfObject::Dictionary(dict) => {
                for (_, value) in &dict.0 {
                    Self::collect_references(value, refs);
                }
            }
            _ => {}
        }
    }
    
    /// Collect references from an object (after resolution)
    fn collect_references_from_object(obj: &PdfObject, refs: &mut Vec<(u32, u16)>) {
        match obj {
            PdfObject::Array(array) => {
                for item in &array.0 {
                    Self::collect_references(item, refs);
                }
            }
            PdfObject::Dictionary(dict) | PdfObject::Stream(PdfStream { dict, .. }) => {
                for (_, value) in &dict.0 {
                    Self::collect_references(value, refs);
                }
            }
            _ => {}
        }
    }
}