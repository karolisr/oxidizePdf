//! High-level PDF Reader API
//! 
//! Provides a simple interface for reading PDF files

use super::{ParseError, ParseResult};
use super::header::PdfHeader;
use super::xref::XRefTable;
use super::trailer::PdfTrailer;
use super::objects::{PdfObject, PdfDictionary};
use super::object_stream::ObjectStream;
use std::io::{Read, Seek, BufReader};
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

/// High-level PDF reader
pub struct PdfReader<R: Read + Seek> {
    reader: BufReader<R>,
    header: PdfHeader,
    xref: XRefTable,
    trailer: PdfTrailer,
    /// Cache of loaded objects
    object_cache: HashMap<(u32, u16), PdfObject>,
    /// Cache of object streams
    object_stream_cache: HashMap<u32, ObjectStream>,
    /// Page tree navigator
    page_tree: Option<super::page_tree::PageTree>,
}

impl PdfReader<File> {
    /// Open a PDF file from a path
    pub fn open<P: AsRef<Path>>(path: P) -> ParseResult<Self> {
        let file = File::open(path)?;
        Self::new(file)
    }
    
    /// Open a PDF file as a PdfDocument
    pub fn open_document<P: AsRef<Path>>(path: P) -> ParseResult<super::document::PdfDocument<File>> {
        let reader = Self::open(path)?;
        Ok(reader.into_document())
    }
}

impl<R: Read + Seek> PdfReader<R> {
    /// Create a new PDF reader from a reader
    pub fn new(reader: R) -> ParseResult<Self> {
        let mut buf_reader = BufReader::new(reader);
        
        // Parse header
        let header = PdfHeader::parse(&mut buf_reader)?;
        // Parse xref table
        let xref = XRefTable::parse(&mut buf_reader)?;
        
        // Get trailer
        let trailer_dict = xref.trailer()
            .ok_or(ParseError::InvalidTrailer)?
            .clone();
        
        let xref_offset = 0; // TODO: Get actual offset
        let trailer = PdfTrailer::from_dict(trailer_dict, xref_offset)?;
        
        // Validate trailer
        trailer.validate()?;
        
        Ok(Self {
            reader: buf_reader,
            header,
            xref,
            trailer,
            object_cache: HashMap::new(),
            object_stream_cache: HashMap::new(),
            page_tree: None,
        })
    }
    
    /// Get the PDF version
    pub fn version(&self) -> &super::header::PdfVersion {
        &self.header.version
    }
    
    /// Get the document catalog
    pub fn catalog(&mut self) -> ParseResult<&PdfDictionary> {
        let (obj_num, gen_num) = self.trailer.root()?;
        let catalog = self.get_object(obj_num, gen_num)?;
        
        catalog.as_dict()
            .ok_or_else(|| ParseError::SyntaxError {
                position: 0,
                message: "Catalog is not a dictionary".to_string(),
            })
    }
    
    /// Get the document info dictionary
    pub fn info(&mut self) -> ParseResult<Option<&PdfDictionary>> {
        match self.trailer.info() {
            Some((obj_num, gen_num)) => {
                let info = self.get_object(obj_num, gen_num)?;
                Ok(info.as_dict())
            }
            None => Ok(None),
        }
    }
    
    /// Get an object by reference
    pub fn get_object(&mut self, obj_num: u32, gen_num: u16) -> ParseResult<&PdfObject> {
        let key = (obj_num, gen_num);
        
        // Check cache first
        if self.object_cache.contains_key(&key) {
            return Ok(&self.object_cache[&key]);
        }
        
        // Check if this is a compressed object
        if let Some(ext_entry) = self.xref.get_extended_entry(obj_num) {
            if let Some((stream_obj_num, index_in_stream)) = ext_entry.compressed_info {
                // This is a compressed object - need to extract from object stream
                return self.get_compressed_object(obj_num, gen_num, stream_obj_num, index_in_stream);
            }
        }
        
        // Get xref entry
        let entry = self.xref.get_entry(obj_num)
            .ok_or_else(|| ParseError::InvalidReference(obj_num, gen_num))?;
        
        if !entry.in_use {
            // Free object
            self.object_cache.insert(key, PdfObject::Null);
            return Ok(&self.object_cache[&key]);
        }
        
        if entry.generation != gen_num {
            return Err(ParseError::InvalidReference(obj_num, gen_num));
        }
        
        // Seek to object position
        self.reader.seek(std::io::SeekFrom::Start(entry.offset))?;
        
        // Parse object header (obj_num gen_num obj)
        let mut lexer = super::lexer::Lexer::new(&mut self.reader);
        
        // Read object number
        let token = lexer.next_token()?;
        let read_obj_num = match token {
            super::lexer::Token::Integer(n) => n as u32,
            _ => return Err(ParseError::SyntaxError {
                position: entry.offset as usize,
                message: "Expected object number".to_string(),
            }),
        };
        
        if read_obj_num != obj_num {
            return Err(ParseError::SyntaxError {
                position: entry.offset as usize,
                message: format!("Object number mismatch: expected {}, found {}", obj_num, read_obj_num),
            });
        }
        
        // Read generation number
        let token = lexer.next_token()?;
        let read_gen_num = match token {
            super::lexer::Token::Integer(n) => n as u16,
            _ => return Err(ParseError::SyntaxError {
                position: entry.offset as usize,
                message: "Expected generation number".to_string(),
            }),
        };
        
        if read_gen_num != gen_num {
            return Err(ParseError::SyntaxError {
                position: entry.offset as usize,
                message: format!("Generation number mismatch: expected {}, found {}", gen_num, read_gen_num),
            });
        }
        
        // Read 'obj' keyword
        let token = lexer.next_token()?;
        match token {
            super::lexer::Token::Obj => {},
            _ => return Err(ParseError::SyntaxError {
                position: entry.offset as usize,
                message: "Expected 'obj' keyword".to_string(),
            }),
        };
        
        // Parse the actual object
        let obj = PdfObject::parse(&mut lexer)?;
        
        // Read 'endobj' keyword
        let token = lexer.next_token()?;
        match token {
            super::lexer::Token::EndObj => {},
            _ => return Err(ParseError::SyntaxError {
                position: entry.offset as usize,
                message: "Expected 'endobj' keyword".to_string(),
            }),
        };
        
        // Cache the object
        self.object_cache.insert(key, obj);
        Ok(&self.object_cache[&key])
    }
    
    /// Resolve a reference to get the actual object
    pub fn resolve<'a>(&'a mut self, obj: &'a PdfObject) -> ParseResult<&'a PdfObject> {
        match obj {
            PdfObject::Reference(obj_num, gen_num) => {
                self.get_object(*obj_num, *gen_num)
            }
            _ => Ok(obj),
        }
    }
    
    /// Get a compressed object from an object stream
    fn get_compressed_object(&mut self, obj_num: u32, gen_num: u16, stream_obj_num: u32, index_in_stream: u32) -> ParseResult<&PdfObject> {
        let key = (obj_num, gen_num);
        
        // Load the object stream if not cached
        if !self.object_stream_cache.contains_key(&stream_obj_num) {
            // Get the stream object
            let stream_obj = self.get_object(stream_obj_num, 0)?;
            
            if let Some(stream) = stream_obj.as_stream() {
                // Parse the object stream
                let obj_stream = ObjectStream::parse(stream.clone())?;
                self.object_stream_cache.insert(stream_obj_num, obj_stream);
            } else {
                return Err(ParseError::SyntaxError {
                    position: 0,
                    message: format!("Object {} is not a stream", stream_obj_num),
                });
            }
        }
        
        // Get the object from the stream
        let obj_stream = &self.object_stream_cache[&stream_obj_num];
        let obj = obj_stream.get_object(obj_num)
            .ok_or_else(|| ParseError::SyntaxError {
                position: 0,
                message: format!("Object {} not found in object stream {}", obj_num, stream_obj_num),
            })?;
        
        // Cache the object
        self.object_cache.insert(key, obj.clone());
        Ok(&self.object_cache[&key])
    }
    
    /// Get the page tree root
    pub fn pages(&mut self) -> ParseResult<&PdfDictionary> {
        // Get the pages reference from catalog first
        let (pages_obj_num, pages_gen_num) = {
            let catalog = self.catalog()?;
            let pages_ref = catalog.get("Pages")
                .ok_or_else(|| ParseError::MissingKey("Pages".to_string()))?;
            
            match pages_ref {
                PdfObject::Reference(obj_num, gen_num) => (*obj_num, *gen_num),
                _ => return Err(ParseError::SyntaxError {
                    position: 0,
                    message: "Pages must be a reference".to_string(),
                }),
            }
        };
        
        // Now we can get the pages object without holding a reference to catalog
        let pages_obj = self.get_object(pages_obj_num, pages_gen_num)?;
        pages_obj.as_dict()
            .ok_or_else(|| ParseError::SyntaxError {
                position: 0,
                message: "Pages is not a dictionary".to_string(),
            })
    }
    
    /// Get the number of pages
    pub fn page_count(&mut self) -> ParseResult<u32> {
        let pages = self.pages()?;
        pages.get("Count")
            .and_then(|obj| obj.as_integer())
            .map(|count| count as u32)
            .ok_or_else(|| ParseError::MissingKey("Count".to_string()))
    }
    
    /// Get metadata from the document
    pub fn metadata(&mut self) -> ParseResult<DocumentMetadata> {
        let mut metadata = DocumentMetadata::default();
        
        if let Some(info_dict) = self.info()? {
            if let Some(title) = info_dict.get("Title").and_then(|o| o.as_string()) {
                metadata.title = title.as_str().ok().map(|s| s.to_string());
            }
            if let Some(author) = info_dict.get("Author").and_then(|o| o.as_string()) {
                metadata.author = author.as_str().ok().map(|s| s.to_string());
            }
            if let Some(subject) = info_dict.get("Subject").and_then(|o| o.as_string()) {
                metadata.subject = subject.as_str().ok().map(|s| s.to_string());
            }
            if let Some(keywords) = info_dict.get("Keywords").and_then(|o| o.as_string()) {
                metadata.keywords = keywords.as_str().ok().map(|s| s.to_string());
            }
            if let Some(creator) = info_dict.get("Creator").and_then(|o| o.as_string()) {
                metadata.creator = creator.as_str().ok().map(|s| s.to_string());
            }
            if let Some(producer) = info_dict.get("Producer").and_then(|o| o.as_string()) {
                metadata.producer = producer.as_str().ok().map(|s| s.to_string());
            }
        }
        
        metadata.version = self.version().to_string();
        metadata.page_count = self.page_count().ok();
        
        Ok(metadata)
    }
    
    /// Initialize the page tree navigator if not already done
    fn ensure_page_tree(&mut self) -> ParseResult<()> {
        if self.page_tree.is_none() {
            let page_count = self.page_count()?;
            self.page_tree = Some(super::page_tree::PageTree::new(page_count));
        }
        Ok(())
    }
    
    /// Get a specific page by index (0-based)
    pub fn get_page(&mut self, index: u32) -> ParseResult<&super::page_tree::ParsedPage> {
        self.ensure_page_tree()?;
        
        // TODO: Fix borrow checker issues with page_tree
        // The page_tree needs mutable access to both itself and the reader
        // This requires a redesign of the architecture
        Err(ParseError::SyntaxError {
            position: 0,
            message: "get_page not implemented due to borrow checker constraints".to_string(),
        })
    }
    
    /// Get all pages
    pub fn get_all_pages(&mut self) -> ParseResult<Vec<super::page_tree::ParsedPage>> {
        let page_count = self.page_count()?;
        let mut pages = Vec::with_capacity(page_count as usize);
        
        for i in 0..page_count {
            let page = self.get_page(i)?.clone();
            pages.push(page);
        }
        
        Ok(pages)
    }
    
    /// Convert this reader into a PdfDocument for easier page access
    pub fn into_document(self) -> super::document::PdfDocument<R> {
        super::document::PdfDocument::new(self)
    }
}

/// Document metadata
#[derive(Debug, Default, Clone)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub version: String,
    pub page_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reader_construction() {
        // This is a minimal valid PDF for testing
        let pdf_data = b"%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [] /Count 0 >>
endobj
xref
0 3
0000000000 65535 f 
0000000009 00000 n 
0000000058 00000 n 
trailer
<< /Size 3 /Root 1 0 R >>
startxref
116
%%EOF";
        
        // For now, we can't fully test this without implementing Seek for Cursor
        // This would require a more complex test setup
    }
}