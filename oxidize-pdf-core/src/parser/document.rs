//! PDF Document wrapper - High-level interface for PDF parsing and manipulation
//!
//! This module provides a robust, high-level interface for working with PDF documents.
//! It solves Rust's borrow checker challenges through careful use of interior mutability
//! (RefCell) and separation of concerns between parsing, caching, and page access.
//!
//! # Architecture
//!
//! The module uses a layered architecture:
//! - **PdfDocument**: Main entry point with RefCell-based state management
//! - **ResourceManager**: Centralized object caching with interior mutability
//! - **PdfReader**: Low-level file access (wrapped in RefCell)
//! - **PageTree**: Lazy-loaded page navigation
//!
//! # Key Features
//!
//! - **Automatic caching**: Objects are cached after first access
//! - **Resource management**: Shared resources are handled efficiently
//! - **Page navigation**: Fast access to any page in the document
//! - **Reference resolution**: Automatic resolution of indirect references
//! - **Text extraction**: Built-in support for extracting text from pages
//!
//! # Example
//!
//! ```rust,no_run
//! use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Open a PDF document
//! let reader = PdfReader::open("document.pdf")?;
//! let document = PdfDocument::new(reader);
//!
//! // Get document information
//! let page_count = document.page_count()?;
//! let metadata = document.metadata()?;
//! println!("Title: {:?}", metadata.title);
//! println!("Pages: {}", page_count);
//!
//! // Access a specific page
//! let page = document.get_page(0)?;
//! println!("Page size: {}x{}", page.width(), page.height());
//!
//! // Extract text from all pages
//! let extracted_text = document.extract_text()?;
//! for (i, page_text) in extracted_text.iter().enumerate() {
//!     println!("Page {}: {}", i + 1, page_text.text);
//! }
//! # Ok(())
//! # }
//! ```

use super::objects::{PdfDictionary, PdfObject};
use super::page_tree::{PageTree, ParsedPage};
use super::reader::PdfReader;
use super::{ParseError, ParseResult};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::rc::Rc;

/// Resource manager for efficient PDF object caching.
///
/// The ResourceManager provides centralized caching of PDF objects to avoid
/// repeated parsing and to share resources between different parts of the document.
/// It uses RefCell for interior mutability, allowing multiple immutable references
/// to the document while still being able to update the cache.
///
/// # Caching Strategy
///
/// - Objects are cached on first access
/// - Cache persists for the lifetime of the document
/// - Manual cache clearing is supported for memory management
///
/// # Example
///
/// ```rust,no_run
/// use oxidize_pdf_core::parser::document::ResourceManager;
///
/// let resources = ResourceManager::new();
///
/// // Objects are cached automatically when accessed through PdfDocument
/// // Manual cache management:
/// resources.clear_cache(); // Free memory when needed
/// ```
pub struct ResourceManager {
    /// Cached objects indexed by (object_number, generation_number)
    object_cache: RefCell<HashMap<(u32, u16), PdfObject>>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            object_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Get an object from cache if available.
    ///
    /// # Arguments
    ///
    /// * `obj_ref` - Object reference (object_number, generation_number)
    ///
    /// # Returns
    ///
    /// Cloned object if cached, None otherwise.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::document::ResourceManager;
    /// # let resources = ResourceManager::new();
    /// if let Some(obj) = resources.get_cached((10, 0)) {
    ///     println!("Object 10 0 R found in cache");
    /// }
    /// ```
    pub fn get_cached(&self, obj_ref: (u32, u16)) -> Option<PdfObject> {
        self.object_cache.borrow().get(&obj_ref).cloned()
    }

    /// Cache an object for future access.
    ///
    /// # Arguments
    ///
    /// * `obj_ref` - Object reference (object_number, generation_number)
    /// * `obj` - The PDF object to cache
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::document::ResourceManager;
    /// # use oxidize_pdf_core::parser::objects::PdfObject;
    /// # let resources = ResourceManager::new();
    /// resources.cache_object((10, 0), PdfObject::Integer(42));
    /// ```
    pub fn cache_object(&self, obj_ref: (u32, u16), obj: PdfObject) {
        self.object_cache.borrow_mut().insert(obj_ref, obj);
    }

    /// Clear all cached objects to free memory.
    ///
    /// Use this when processing large documents to manage memory usage.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::document::ResourceManager;
    /// # let resources = ResourceManager::new();
    /// // After processing many pages
    /// resources.clear_cache();
    /// println!("Cache cleared to free memory");
    /// ```
    pub fn clear_cache(&self) {
        self.object_cache.borrow_mut().clear();
    }
}

/// High-level PDF document interface for parsing and manipulation.
///
/// `PdfDocument` provides a clean, safe API for working with PDF files.
/// It handles the complexity of PDF structure, object references, and resource
/// management behind a simple interface.
///
/// # Type Parameter
///
/// * `R` - The reader type (must implement Read + Seek)
///
/// # Architecture Benefits
///
/// - **RefCell Usage**: Allows multiple parts of the API to access the document
/// - **Lazy Loading**: Pages and resources are loaded on demand
/// - **Automatic Caching**: Frequently accessed objects are cached
/// - **Safe API**: Borrow checker issues are handled internally
///
/// # Example
///
/// ```rust,no_run
/// use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
/// use std::fs::File;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // From a file
/// let reader = PdfReader::open("document.pdf")?;
/// let document = PdfDocument::new(reader);
///
/// // From any Read + Seek source
/// let file = File::open("document.pdf")?;
/// let reader = PdfReader::new(file)?;
/// let document = PdfDocument::new(reader);
///
/// // Use the document
/// let page_count = document.page_count()?;
/// for i in 0..page_count {
///     let page = document.get_page(i)?;
///     // Process page...
/// }
/// # Ok(())
/// # }
/// ```
pub struct PdfDocument<R: Read + Seek> {
    /// The underlying PDF reader wrapped for interior mutability
    reader: RefCell<PdfReader<R>>,
    /// Page tree navigator (lazily initialized)
    page_tree: RefCell<Option<PageTree>>,
    /// Shared resource manager for object caching
    resources: Rc<ResourceManager>,
    /// Cached document metadata to avoid repeated parsing
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

    /// Get the PDF version of the document.
    ///
    /// # Returns
    ///
    /// PDF version string (e.g., "1.4", "1.7", "2.0")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let reader = PdfReader::open("document.pdf")?;
    /// # let document = PdfDocument::new(reader);
    /// let version = document.version()?;
    /// println!("PDF version: {}", version);
    /// # Ok(())
    /// # }
    /// ```
    pub fn version(&self) -> ParseResult<String> {
        Ok(self.reader.borrow().version().to_string())
    }

    /// Get the total number of pages in the document.
    ///
    /// # Returns
    ///
    /// The page count as an unsigned 32-bit integer.
    ///
    /// # Errors
    ///
    /// Returns an error if the page tree is malformed or missing.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let reader = PdfReader::open("document.pdf")?;
    /// # let document = PdfDocument::new(reader);
    /// let count = document.page_count()?;
    /// println!("Document has {} pages", count);
    ///
    /// // Iterate through all pages
    /// for i in 0..count {
    ///     let page = document.get_page(i)?;
    ///     // Process page...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn page_count(&self) -> ParseResult<u32> {
        self.reader.borrow_mut().page_count()
    }

    /// Get document metadata including title, author, creation date, etc.
    ///
    /// Metadata is cached after first access for performance.
    ///
    /// # Returns
    ///
    /// A `DocumentMetadata` struct containing all available metadata fields.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let reader = PdfReader::open("document.pdf")?;
    /// # let document = PdfDocument::new(reader);
    /// let metadata = document.metadata()?;
    ///
    /// if let Some(title) = &metadata.title {
    ///     println!("Title: {}", title);
    /// }
    /// if let Some(author) = &metadata.author {
    ///     println!("Author: {}", author);
    /// }
    /// if let Some(creation_date) = &metadata.creation_date {
    ///     println!("Created: {}", creation_date);
    /// }
    /// println!("PDF Version: {}", metadata.version);
    /// # Ok(())
    /// # }
    /// ```
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

    /// Get a page by index (0-based).
    ///
    /// Pages are cached after first access. This method handles page tree
    /// traversal and property inheritance automatically.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based page index (0 to page_count-1)
    ///
    /// # Returns
    ///
    /// A complete `ParsedPage` with all properties and inherited resources.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Index is out of bounds
    /// - Page tree is malformed
    /// - Required page properties are missing
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let reader = PdfReader::open("document.pdf")?;
    /// # let document = PdfDocument::new(reader);
    /// // Get the first page
    /// let page = document.get_page(0)?;
    ///
    /// // Access page properties
    /// println!("Page size: {}x{} points", page.width(), page.height());
    /// println!("Rotation: {}°", page.rotation);
    ///
    /// // Get content streams
    /// let streams = page.content_streams_with_document(&document)?;
    /// println!("Page has {} content streams", streams.len());
    /// # Ok(())
    /// # }
    /// ```
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
        let node_type = node
            .get_type()
            .ok_or_else(|| ParseError::MissingKey("Type".to_string()))?;

        match node_type {
            "Pages" => {
                // This is a page tree node
                let kids = node
                    .get("Kids")
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
                    let kid_ref =
                        kid_ref
                            .as_reference()
                            .ok_or_else(|| ParseError::SyntaxError {
                                position: 0,
                                message: "Kids array must contain references".to_string(),
                            })?;

                    // Get the kid object
                    let kid_obj = self.get_object(kid_ref.0, kid_ref.1)?;
                    let kid_dict = kid_obj.as_dict().ok_or_else(|| ParseError::SyntaxError {
                        position: 0,
                        message: "Page tree node must be a dictionary".to_string(),
                    })?;

                    let kid_type = kid_dict
                        .get_type()
                        .ok_or_else(|| ParseError::MissingKey("Type".to_string()))?;

                    let count = if kid_type == "Pages" {
                        kid_dict
                            .get("Count")
                            .and_then(|obj| obj.as_integer())
                            .ok_or_else(|| ParseError::MissingKey("Count".to_string()))?
                            as u32
                    } else {
                        1
                    };

                    if target_index < current_idx + count {
                        // Found the right subtree/page
                        if kid_type == "Page" {
                            // This is the page we want
                            return self.create_parsed_page(
                                kid_ref,
                                kid_dict,
                                Some(&merged_inherited),
                            );
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
                message: format!("Invalid page tree node type: {node_type}"),
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
        let media_box = self
            .get_rectangle(page_dict, inherited, "MediaBox")?
            .ok_or_else(|| ParseError::MissingKey("MediaBox".to_string()))?;

        let crop_box = self.get_rectangle(page_dict, inherited, "CropBox")?;

        let rotation = self
            .get_integer(page_dict, inherited, "Rotate")?
            .unwrap_or(0) as i32;

        // Get inherited resources
        let inherited_resources = if let Some(inherited) = inherited {
            inherited
                .get("Resources")
                .and_then(|r| r.as_dict())
                .cloned()
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
        let array = node.get(key).or_else(|| inherited.and_then(|i| i.get(key)));

        if let Some(array) = array.and_then(|obj| obj.as_array()) {
            if array.len() != 4 {
                return Err(ParseError::SyntaxError {
                    position: 0,
                    message: format!("{key} must have 4 elements"),
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
        let value = node.get(key).or_else(|| inherited.and_then(|i| i.get(key)));

        Ok(value.and_then(|obj| obj.as_integer()))
    }

    /// Get an object by its reference numbers.
    ///
    /// This method first checks the cache, then loads from the file if needed.
    /// Objects are automatically cached after loading.
    ///
    /// # Arguments
    ///
    /// * `obj_num` - Object number
    /// * `gen_num` - Generation number
    ///
    /// # Returns
    ///
    /// The resolved PDF object.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Object doesn't exist
    /// - Object is part of an encrypted object stream
    /// - File is corrupted
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let reader = PdfReader::open("document.pdf")?;
    /// # let document = PdfDocument::new(reader);
    /// // Get object 10 0 R
    /// let obj = document.get_object(10, 0)?;
    ///
    /// // Check object type
    /// match obj {
    ///     PdfObject::Dictionary(dict) => {
    ///         println!("Object is a dictionary with {} entries", dict.0.len());
    ///     }
    ///     PdfObject::Stream(stream) => {
    ///         println!("Object is a stream");
    ///     }
    ///     _ => {}
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

    /// Resolve a reference to get the actual object.
    ///
    /// If the input is a Reference, fetches the referenced object.
    /// Otherwise returns a clone of the input object.
    ///
    /// # Arguments
    ///
    /// * `obj` - The object to resolve (may be a Reference or direct object)
    ///
    /// # Returns
    ///
    /// The resolved object (never a Reference).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
    /// # use oxidize_pdf_core::parser::objects::PdfObject;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let reader = PdfReader::open("document.pdf")?;
    /// # let document = PdfDocument::new(reader);
    /// # let page = document.get_page(0)?;
    /// // Contents might be a reference or direct object
    /// if let Some(contents) = page.dict.get("Contents") {
    ///     let resolved = document.resolve(contents)?;
    ///     match resolved {
    ///         PdfObject::Stream(_) => println!("Single content stream"),
    ///         PdfObject::Array(_) => println!("Multiple content streams"),
    ///         _ => println!("Unexpected content type"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn resolve(&self, obj: &PdfObject) -> ParseResult<PdfObject> {
        match obj {
            PdfObject::Reference(obj_num, gen_num) => self.get_object(*obj_num, *gen_num),
            _ => Ok(obj.clone()),
        }
    }

    /// Get content streams for a specific page.
    ///
    /// This method handles both single streams and arrays of streams,
    /// automatically decompressing them according to their filters.
    ///
    /// # Arguments
    ///
    /// * `page` - The page to get content streams from
    ///
    /// # Returns
    ///
    /// Vector of decompressed content stream data ready for parsing.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
    /// # use oxidize_pdf_core::parser::content::ContentParser;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let reader = PdfReader::open("document.pdf")?;
    /// # let document = PdfDocument::new(reader);
    /// let page = document.get_page(0)?;
    /// let streams = document.get_page_content_streams(&page)?;
    ///
    /// // Parse content streams
    /// for stream_data in streams {
    ///     let operations = ContentParser::parse(&stream_data)?;
    ///     println!("Stream has {} operations", operations.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
                _ => {
                    return Err(ParseError::SyntaxError {
                        position: 0,
                        message: "Contents must be a stream or array of streams".to_string(),
                    })
                }
            }
        }

        Ok(streams)
    }

    /// Extract text from all pages in the document.
    ///
    /// Uses the default text extraction settings. For custom settings,
    /// use `extract_text_with_options`.
    ///
    /// # Returns
    ///
    /// A vector of `ExtractedText`, one for each page in the document.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let reader = PdfReader::open("document.pdf")?;
    /// # let document = PdfDocument::new(reader);
    /// let extracted_pages = document.extract_text()?;
    ///
    /// for (page_num, page_text) in extracted_pages.iter().enumerate() {
    ///     println!("=== Page {} ===", page_num + 1);
    ///     println!("{}", page_text.text);
    ///     println!();
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn extract_text(&self) -> ParseResult<Vec<crate::text::ExtractedText>> {
        let extractor = crate::text::TextExtractor::new();
        extractor.extract_from_document(self)
    }

    /// Extract text from a specific page.
    ///
    /// # Arguments
    ///
    /// * `page_index` - Zero-based page index
    ///
    /// # Returns
    ///
    /// Extracted text with optional position information.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let reader = PdfReader::open("document.pdf")?;
    /// # let document = PdfDocument::new(reader);
    /// // Extract text from first page only
    /// let page_text = document.extract_text_from_page(0)?;
    /// println!("First page text: {}", page_text.text);
    ///
    /// // Access text fragments with positions (if preserved)
    /// for fragment in &page_text.fragments {
    ///     println!("'{}' at ({}, {})", fragment.text, fragment.x, fragment.y);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn extract_text_from_page(
        &self,
        page_index: u32,
    ) -> ParseResult<crate::text::ExtractedText> {
        let extractor = crate::text::TextExtractor::new();
        extractor.extract_from_page(self, page_index)
    }

    /// Extract text with custom extraction options.
    ///
    /// Allows fine control over text extraction behavior including
    /// layout preservation, spacing thresholds, and more.
    ///
    /// # Arguments
    ///
    /// * `options` - Text extraction configuration
    ///
    /// # Returns
    ///
    /// A vector of `ExtractedText`, one for each page.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::{PdfDocument, PdfReader};
    /// # use oxidize_pdf_core::text::ExtractionOptions;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let reader = PdfReader::open("document.pdf")?;
    /// # let document = PdfDocument::new(reader);
    /// // Configure extraction to preserve layout
    /// let options = ExtractionOptions {
    ///     preserve_layout: true,
    ///     space_threshold: 0.3,
    ///     newline_threshold: 10.0,
    /// };
    ///
    /// let extracted_pages = document.extract_text_with_options(options)?;
    ///
    /// // Text fragments will include position information
    /// for page_text in extracted_pages {
    ///     for fragment in &page_text.fragments {
    ///         println!("{:?}", fragment);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn extract_text_with_options(
        &self,
        options: crate::text::ExtractionOptions,
    ) -> ParseResult<Vec<crate::text::ExtractedText>> {
        let extractor = crate::text::TextExtractor::with_options(options);
        extractor.extract_from_document(self)
    }
}
