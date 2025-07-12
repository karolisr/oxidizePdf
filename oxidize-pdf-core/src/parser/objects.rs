//! PDF Object Parser - Core PDF data types and parsing
//!
//! This module implements parsing of all PDF object types according to ISO 32000-1 Section 7.3.
//! PDF files are built from a small set of basic object types that can be combined to form
//! complex data structures.
//!
//! # Object Types
//!
//! PDF supports the following basic object types:
//! - **Null**: Represents an undefined value
//! - **Boolean**: true or false
//! - **Integer**: Whole numbers
//! - **Real**: Floating-point numbers
//! - **String**: Text data (literal or hexadecimal)
//! - **Name**: Unique atomic symbols (e.g., /Type, /Pages)
//! - **Array**: Ordered collections of objects
//! - **Dictionary**: Key-value mappings where keys are names
//! - **Stream**: Dictionary + binary data
//! - **Reference**: Indirect reference to another object
//!
//! # Example
//!
//! ```rust
//! use oxidize_pdf_core::parser::objects::{PdfObject, PdfDictionary, PdfName};
//!
//! // Create a simple page dictionary
//! let mut dict = PdfDictionary::new();
//! dict.insert("Type".to_string(), PdfObject::Name(PdfName::new("Page".to_string())));
//! dict.insert("MediaBox".to_string(), PdfObject::Array(PdfArray::new()));
//!
//! // Check dictionary type
//! assert_eq!(dict.get_type(), Some("Page"));
//! ```

use super::lexer::{Lexer, Token};
use super::{ParseError, ParseResult};
use std::collections::HashMap;
use std::io::Read;

/// PDF Name object - Unique atomic symbols in PDF.
///
/// Names are used as keys in dictionaries and to identify various PDF constructs.
/// They are written with a leading slash (/) in PDF syntax but stored without it.
///
/// # Examples
///
/// Common PDF names:
/// - `/Type` - Object type identifier
/// - `/Pages` - Page tree root
/// - `/Font` - Font resource
/// - `/MediaBox` - Page dimensions
///
/// ```rust
/// use oxidize_pdf_core::parser::objects::PdfName;
///
/// let name = PdfName::new("Type".to_string());
/// assert_eq!(name.as_str(), "Type");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PdfName(pub String);

/// PDF String object - Text data in PDF files.
///
/// PDF strings can contain arbitrary binary data and use various encodings.
/// They can be written as literal strings `(text)` or hexadecimal strings `<48656C6C6F>`.
///
/// # Encoding
///
/// String encoding depends on context:
/// - Text strings: Usually PDFDocEncoding or UTF-16BE
/// - Font strings: Encoding specified by the font
/// - Binary data: No encoding, raw bytes
///
/// # Example
///
/// ```rust
/// use oxidize_pdf_core::parser::objects::PdfString;
///
/// // Create from UTF-8
/// let string = PdfString::new(b"Hello World".to_vec());
/// 
/// // Try to decode as UTF-8
/// if let Ok(text) = string.as_str() {
///     println!("Text: {}", text);
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PdfString(pub Vec<u8>);

/// PDF Array object - Ordered collection of PDF objects.
///
/// Arrays can contain any PDF object type, including other arrays and dictionaries.
/// They are written in PDF syntax as `[item1 item2 ... itemN]`.
///
/// # Common Uses
///
/// - Rectangle specifications: `[llx lly urx ury]`
/// - Color values: `[r g b]`
/// - Matrix transformations: `[a b c d e f]`
/// - Resource lists
///
/// # Example
///
/// ```rust
/// use oxidize_pdf_core::parser::objects::{PdfArray, PdfObject};
///
/// // Create a MediaBox array [0 0 612 792]
/// let mut media_box = PdfArray::new();
/// media_box.push(PdfObject::Integer(0));
/// media_box.push(PdfObject::Integer(0));
/// media_box.push(PdfObject::Integer(612));
/// media_box.push(PdfObject::Integer(792));
///
/// assert_eq!(media_box.len(), 4);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PdfArray(pub Vec<PdfObject>);

/// PDF Dictionary object - Key-value mapping with name keys.
///
/// Dictionaries are the primary way to represent complex data structures in PDF.
/// Keys must be PdfName objects, values can be any PDF object type.
///
/// # Common Dictionary Types
///
/// - **Catalog**: Document root (`/Type /Catalog`)
/// - **Page**: Individual page (`/Type /Page`)
/// - **Font**: Font definition (`/Type /Font`)
/// - **Stream**: Binary data with metadata
///
/// # Example
///
/// ```rust
/// use oxidize_pdf_core::parser::objects::{PdfDictionary, PdfObject, PdfName};
///
/// let mut page_dict = PdfDictionary::new();
/// page_dict.insert("Type".to_string(), 
///     PdfObject::Name(PdfName::new("Page".to_string())));
/// page_dict.insert("Parent".to_string(), 
///     PdfObject::Reference(2, 0)); // Reference to pages tree
///
/// // Access values
/// assert_eq!(page_dict.get_type(), Some("Page"));
/// assert!(page_dict.contains_key("Parent"));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PdfDictionary(pub HashMap<PdfName, PdfObject>);

/// PDF Stream object - Dictionary with associated binary data.
///
/// Streams are used for large data blocks like page content, images, fonts, etc.
/// The dictionary describes the stream's properties (length, filters, etc.).
///
/// # Structure
///
/// - `dict`: Stream dictionary with metadata
/// - `data`: Raw stream bytes (possibly compressed)
///
/// # Common Stream Types
///
/// - **Content streams**: Page drawing instructions
/// - **Image XObjects**: Embedded images
/// - **Font programs**: Embedded font data
/// - **Form XObjects**: Reusable graphics
///
/// # Example
///
/// ```rust
/// use oxidize_pdf_core::parser::objects::{PdfStream, PdfDictionary};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let stream = PdfStream { dict: PdfDictionary::new(), data: vec![] };
/// // Get decompressed data
/// let decoded = stream.decode()?;
/// println!("Decoded {} bytes", decoded.len());
///
/// // Access raw data
/// let raw = stream.raw_data();
/// println!("Raw {} bytes", raw.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PdfStream {
    /// Stream dictionary containing Length, Filter, and other properties
    pub dict: PdfDictionary,
    /// Raw stream data (may be compressed)
    pub data: Vec<u8>,
}

impl PdfStream {
    /// Get the decompressed stream data.
    ///
    /// Automatically applies filters specified in the stream dictionary
    /// (FlateDecode, ASCIIHexDecode, etc.) to decompress the data.
    ///
    /// # Returns
    ///
    /// The decoded/decompressed stream bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Unknown filter is specified
    /// - Decompression fails
    /// - Filter parameters are invalid
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf_core::parser::objects::PdfStream;
    /// # fn example(stream: &PdfStream) -> Result<(), Box<dyn std::error::Error>> {
    /// match stream.decode() {
    ///     Ok(data) => println!("Decoded {} bytes", data.len()),
    ///     Err(e) => println!("Decode error: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn decode(&self) -> ParseResult<Vec<u8>> {
        super::filters::decode_stream(&self.data, &self.dict)
    }

    /// Get the raw (possibly compressed) stream data.
    ///
    /// Returns the stream data exactly as stored in the PDF file,
    /// without applying any filters or decompression.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use oxidize_pdf_core::parser::objects::PdfStream;
    /// # let stream = PdfStream { dict: Default::default(), data: vec![1, 2, 3] };
    /// let raw_data = stream.raw_data();
    /// println!("Raw stream: {} bytes", raw_data.len());
    /// ```
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }
}

/// PDF Object types - The fundamental data types in PDF.
///
/// All data in a PDF file is represented using these basic types.
/// Objects can be direct (embedded) or indirect (referenced).
///
/// # Object Types
///
/// - `Null` - Undefined/absent value
/// - `Boolean` - true or false
/// - `Integer` - Signed integers
/// - `Real` - Floating-point numbers
/// - `String` - Text or binary data
/// - `Name` - Atomic symbols like /Type
/// - `Array` - Ordered collections
/// - `Dictionary` - Key-value maps
/// - `Stream` - Dictionary + binary data
/// - `Reference` - Indirect object reference (num gen R)
///
/// # Example
///
/// ```rust
/// use oxidize_pdf_core::parser::objects::{PdfObject, PdfName, PdfString};
///
/// // Different object types
/// let null = PdfObject::Null;
/// let bool_val = PdfObject::Boolean(true);
/// let int_val = PdfObject::Integer(42);
/// let real_val = PdfObject::Real(3.14159);
/// let name = PdfObject::Name(PdfName::new("Type".to_string()));
/// let reference = PdfObject::Reference(10, 0); // 10 0 R
///
/// // Type checking
/// assert!(int_val.as_integer().is_some());
/// assert_eq!(int_val.as_integer(), Some(42));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum PdfObject {
    /// Null object - represents undefined or absent values
    Null,
    /// Boolean value - true or false
    Boolean(bool),
    /// Integer number
    Integer(i64),
    /// Real (floating-point) number
    Real(f64),
    /// String data (literal or hexadecimal)
    String(PdfString),
    /// Name object - unique identifier
    Name(PdfName),
    /// Array - ordered collection of objects
    Array(PdfArray),
    /// Dictionary - unordered key-value pairs
    Dictionary(PdfDictionary),
    /// Stream - dictionary with binary data
    Stream(PdfStream),
    /// Indirect object reference (object_number, generation_number)
    Reference(u32, u16),
}

impl PdfObject {
    /// Parse a PDF object from a lexer.
    ///
    /// Reads tokens from the lexer and constructs the appropriate PDF object.
    /// Handles all PDF object types including indirect references.
    ///
    /// # Arguments
    ///
    /// * `lexer` - Token source for parsing
    ///
    /// # Returns
    ///
    /// The parsed PDF object.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Invalid syntax is encountered
    /// - Unexpected end of input
    /// - Malformed object structure
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use oxidize_pdf_core::parser::lexer::Lexer;
    /// use oxidize_pdf_core::parser::objects::PdfObject;
    /// use std::io::Cursor;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let input = b"42";
    /// let mut lexer = Lexer::new(Cursor::new(input));
    /// let obj = PdfObject::parse(&mut lexer)?;
    /// assert_eq!(obj, PdfObject::Integer(42));
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse<R: Read>(lexer: &mut Lexer<R>) -> ParseResult<Self> {
        let token = lexer.next_token()?;
        Self::parse_from_token(lexer, token)
    }

    /// Parse a PDF object starting from a specific token
    fn parse_from_token<R: Read>(lexer: &mut Lexer<R>, token: Token) -> ParseResult<Self> {
        match token {
            Token::Null => Ok(PdfObject::Null),
            Token::Boolean(b) => Ok(PdfObject::Boolean(b)),
            Token::Integer(i) => {
                // For negative numbers or large values, don't check for references
                if !(0..=9999999).contains(&i) {
                    return Ok(PdfObject::Integer(i));
                }

                // Check if this is part of a reference (e.g., "1 0 R")
                match lexer.next_token()? {
                    Token::Integer(gen) if (0..=65535).contains(&gen) => {
                        // Might be a reference, check for 'R'
                        match lexer.next_token()? {
                            Token::Name(s) if s == "R" => {
                                Ok(PdfObject::Reference(i as u32, gen as u16))
                            }
                            token => {
                                // Not a reference, push back the tokens
                                lexer.push_token(token);
                                lexer.push_token(Token::Integer(gen));
                                Ok(PdfObject::Integer(i))
                            }
                        }
                    }
                    token => {
                        // Not a reference, just an integer
                        lexer.push_token(token);
                        Ok(PdfObject::Integer(i))
                    }
                }
            }
            Token::Real(r) => Ok(PdfObject::Real(r)),
            Token::String(s) => Ok(PdfObject::String(PdfString(s))),
            Token::Name(n) => Ok(PdfObject::Name(PdfName(n))),
            Token::ArrayStart => Self::parse_array(lexer),
            Token::DictStart => Self::parse_dictionary_or_stream(lexer),
            Token::Comment(_) => {
                // Skip comments and parse next object
                Self::parse(lexer)
            }
            Token::StartXRef => {
                // This is a PDF structure marker, not a parseable object
                Err(ParseError::SyntaxError {
                    position: 0,
                    message: "StartXRef encountered - this is not a PDF object".to_string(),
                })
            }
            Token::Eof => Err(ParseError::SyntaxError {
                position: 0,
                message: "Unexpected end of file".to_string(),
            }),
            _ => Err(ParseError::UnexpectedToken {
                expected: "PDF object".to_string(),
                found: format!("{token:?}"),
            }),
        }
    }

    /// Parse a PDF array
    fn parse_array<R: Read>(lexer: &mut Lexer<R>) -> ParseResult<Self> {
        let mut elements = Vec::new();

        loop {
            let token = lexer.next_token()?;
            match token {
                Token::ArrayEnd => break,
                Token::Comment(_) => continue, // Skip comments
                _ => {
                    let obj = Self::parse_from_token(lexer, token)?;
                    elements.push(obj);
                }
            }
        }

        Ok(PdfObject::Array(PdfArray(elements)))
    }

    /// Parse a PDF dictionary and check if it's followed by a stream
    fn parse_dictionary_or_stream<R: Read>(lexer: &mut Lexer<R>) -> ParseResult<Self> {
        let dict = Self::parse_dictionary_inner(lexer)?;

        // Check if this is followed by a stream
        loop {
            let token = lexer.next_token()?;
            // Check for stream
            match token {
                Token::Stream => {
                    // Parse stream data
                    let stream_data = Self::parse_stream_data(lexer, &dict)?;
                    return Ok(PdfObject::Stream(PdfStream {
                        dict,
                        data: stream_data,
                    }));
                }
                Token::Comment(_) => {
                    // Skip comment and continue checking
                    continue;
                }
                Token::StartXRef => {
                    // This is the end of the PDF structure, not a stream
                    // Push the token back for later processing
                    // Push back StartXRef token
                    lexer.push_token(token);
                    return Ok(PdfObject::Dictionary(dict));
                }
                _ => {
                    // Not a stream, just a dictionary
                    // Push the token back for later processing
                    // Push back token
                    lexer.push_token(token);
                    return Ok(PdfObject::Dictionary(dict));
                }
            }
        }
    }

    /// Parse the inner dictionary
    fn parse_dictionary_inner<R: Read>(lexer: &mut Lexer<R>) -> ParseResult<PdfDictionary> {
        let mut dict = HashMap::new();

        loop {
            let token = lexer.next_token()?;
            match token {
                Token::DictEnd => break,
                Token::Comment(_) => continue, // Skip comments
                Token::Name(key) => {
                    let value = Self::parse(lexer)?;
                    dict.insert(PdfName(key), value);
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "dictionary key (name) or >>".to_string(),
                        found: format!("{token:?}"),
                    });
                }
            }
        }

        Ok(PdfDictionary(dict))
    }

    /// Parse stream data
    fn parse_stream_data<R: Read>(
        lexer: &mut Lexer<R>,
        dict: &PdfDictionary,
    ) -> ParseResult<Vec<u8>> {
        // Get the stream length from the dictionary
        let length = dict
            .0
            .get(&PdfName("Length".to_string()))
            .ok_or_else(|| ParseError::MissingKey("Length".to_string()))?;

        let length = match length {
            PdfObject::Integer(len) => *len as usize,
            PdfObject::Reference(_, _) => {
                // In a full implementation, we'd need to resolve this reference
                // For now, we'll return an error
                return Err(ParseError::SyntaxError {
                    position: lexer.position(),
                    message: "Stream length references not yet supported".to_string(),
                });
            }
            _ => {
                return Err(ParseError::SyntaxError {
                    position: lexer.position(),
                    message: "Invalid stream length type".to_string(),
                });
            }
        };

        // Skip the newline after 'stream' keyword
        lexer.read_newline()?;

        // Read the actual stream data
        let stream_data = lexer.read_bytes(length)?;

        // Skip optional whitespace before endstream
        lexer.skip_whitespace()?;

        // Read 'endstream' keyword
        let token = lexer.next_token()?;
        match token {
            Token::EndStream => Ok(stream_data),
            _ => Err(ParseError::UnexpectedToken {
                expected: "endstream".to_string(),
                found: format!("{token:?}"),
            }),
        }
    }

    /// Check if this object is null.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxidize_pdf_core::parser::objects::PdfObject;
    ///
    /// assert!(PdfObject::Null.is_null());
    /// assert!(!PdfObject::Integer(42).is_null());
    /// ```
    pub fn is_null(&self) -> bool {
        matches!(self, PdfObject::Null)
    }

    /// Get the value as a boolean if this is a Boolean object.
    ///
    /// # Returns
    ///
    /// Some(bool) if this is a Boolean object, None otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxidize_pdf_core::parser::objects::PdfObject;
    ///
    /// let obj = PdfObject::Boolean(true);
    /// assert_eq!(obj.as_bool(), Some(true));
    ///
    /// let obj = PdfObject::Integer(1);
    /// assert_eq!(obj.as_bool(), None);
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PdfObject::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            PdfObject::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get the value as a real number.
    ///
    /// Returns the value for both Real and Integer objects,
    /// converting integers to floating-point.
    ///
    /// # Returns
    ///
    /// Some(f64) if this is a numeric object, None otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxidize_pdf_core::parser::objects::PdfObject;
    ///
    /// let real_obj = PdfObject::Real(3.14);
    /// assert_eq!(real_obj.as_real(), Some(3.14));
    ///
    /// let int_obj = PdfObject::Integer(42);
    /// assert_eq!(int_obj.as_real(), Some(42.0));
    /// ```
    pub fn as_real(&self) -> Option<f64> {
        match self {
            PdfObject::Real(r) => Some(*r),
            PdfObject::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Get as string
    pub fn as_string(&self) -> Option<&PdfString> {
        match self {
            PdfObject::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as name
    pub fn as_name(&self) -> Option<&PdfName> {
        match self {
            PdfObject::Name(n) => Some(n),
            _ => None,
        }
    }

    /// Get as array
    pub fn as_array(&self) -> Option<&PdfArray> {
        match self {
            PdfObject::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Get as dictionary
    pub fn as_dict(&self) -> Option<&PdfDictionary> {
        match self {
            PdfObject::Dictionary(d) => Some(d),
            PdfObject::Stream(s) => Some(&s.dict),
            _ => None,
        }
    }

    /// Get as stream
    pub fn as_stream(&self) -> Option<&PdfStream> {
        match self {
            PdfObject::Stream(s) => Some(s),
            _ => None,
        }
    }

    /// Get the object reference if this is a Reference object.
    ///
    /// # Returns
    ///
    /// Some((object_number, generation_number)) if this is a Reference, None otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxidize_pdf_core::parser::objects::PdfObject;
    ///
    /// let obj = PdfObject::Reference(10, 0);
    /// assert_eq!(obj.as_reference(), Some((10, 0)));
    ///
    /// // Use for resolving references
    /// if let Some((obj_num, gen_num)) = obj.as_reference() {
    ///     println!("Reference to {} {} R", obj_num, gen_num);
    /// }
    /// ```
    pub fn as_reference(&self) -> Option<(u32, u16)> {
        match self {
            PdfObject::Reference(obj, gen) => Some((*obj, *gen)),
            _ => None,
        }
    }
}

impl Default for PdfDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfDictionary {
    /// Create a new empty dictionary.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxidize_pdf_core::parser::objects::{PdfDictionary, PdfObject, PdfName};
    ///
    /// let mut dict = PdfDictionary::new();
    /// dict.insert("Type".to_string(), PdfObject::Name(PdfName::new("Font".to_string())));
    /// ```
    pub fn new() -> Self {
        PdfDictionary(HashMap::new())
    }

    /// Get a value by key name.
    ///
    /// # Arguments
    ///
    /// * `key` - The key name (without leading slash)
    ///
    /// # Returns
    ///
    /// Reference to the value if the key exists, None otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxidize_pdf_core::parser::objects::{PdfDictionary, PdfObject};
    ///
    /// let mut dict = PdfDictionary::new();
    /// dict.insert("Length".to_string(), PdfObject::Integer(1000));
    ///
    /// if let Some(length) = dict.get("Length").and_then(|o| o.as_integer()) {
    ///     println!("Stream length: {}", length);
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<&PdfObject> {
        self.0.get(&PdfName(key.to_string()))
    }

    /// Insert a key-value pair
    pub fn insert(&mut self, key: String, value: PdfObject) {
        self.0.insert(PdfName(key), value);
    }

    /// Check if dictionary contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(&PdfName(key.to_string()))
    }

    /// Get the dictionary type (value of /Type key).
    ///
    /// Many PDF dictionaries have a /Type entry that identifies their purpose.
    ///
    /// # Returns
    ///
    /// The type name if present, None otherwise.
    ///
    /// # Common Types
    ///
    /// - "Catalog" - Document catalog
    /// - "Page" - Page object
    /// - "Pages" - Page tree node
    /// - "Font" - Font dictionary
    /// - "XObject" - External object
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxidize_pdf_core::parser::objects::{PdfDictionary, PdfObject, PdfName};
    ///
    /// let mut dict = PdfDictionary::new();
    /// dict.insert("Type".to_string(), PdfObject::Name(PdfName::new("Page".to_string())));
    /// assert_eq!(dict.get_type(), Some("Page"));
    /// ```
    pub fn get_type(&self) -> Option<&str> {
        self.get("Type")
            .and_then(|obj| obj.as_name())
            .map(|n| n.0.as_str())
    }
}

impl Default for PdfArray {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfArray {
    /// Create a new empty array
    pub fn new() -> Self {
        PdfArray(Vec::new())
    }

    /// Get array length
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if array is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get element at index.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index
    ///
    /// # Returns
    ///
    /// Reference to the element if index is valid, None otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxidize_pdf_core::parser::objects::{PdfArray, PdfObject};
    ///
    /// let mut array = PdfArray::new();
    /// array.push(PdfObject::Integer(10));
    /// array.push(PdfObject::Integer(20));
    ///
    /// assert_eq!(array.get(0).and_then(|o| o.as_integer()), Some(10));
    /// assert_eq!(array.get(1).and_then(|o| o.as_integer()), Some(20));
    /// assert!(array.get(2).is_none());
    /// ```
    pub fn get(&self, index: usize) -> Option<&PdfObject> {
        self.0.get(index)
    }

    /// Push an element
    pub fn push(&mut self, obj: PdfObject) {
        self.0.push(obj);
    }
}

impl PdfString {
    /// Create a new PDF string
    pub fn new(data: Vec<u8>) -> Self {
        PdfString(data)
    }

    /// Get as UTF-8 string if possible.
    ///
    /// Attempts to decode the string bytes as UTF-8.
    /// Note that PDF strings may use other encodings.
    ///
    /// # Returns
    ///
    /// Ok(&str) if valid UTF-8, Err otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxidize_pdf_core::parser::objects::PdfString;
    ///
    /// let string = PdfString::new(b"Hello".to_vec());
    /// assert_eq!(string.as_str(), Ok("Hello"));
    ///
    /// let binary = PdfString::new(vec![0xFF, 0xFE]);
    /// assert!(binary.as_str().is_err());
    /// ```
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl PdfName {
    /// Create a new PDF name
    pub fn new(name: String) -> Self {
        PdfName(name)
    }

    /// Get the name as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_simple_objects() {
        let input = b"null true false 123 -456 3.14 /Name (Hello)";
        let mut lexer = Lexer::new(Cursor::new(input));

        assert_eq!(PdfObject::parse(&mut lexer).unwrap(), PdfObject::Null);
        assert_eq!(
            PdfObject::parse(&mut lexer).unwrap(),
            PdfObject::Boolean(true)
        );
        assert_eq!(
            PdfObject::parse(&mut lexer).unwrap(),
            PdfObject::Boolean(false)
        );
        assert_eq!(
            PdfObject::parse(&mut lexer).unwrap(),
            PdfObject::Integer(123)
        );
        assert_eq!(
            PdfObject::parse(&mut lexer).unwrap(),
            PdfObject::Integer(-456)
        );
        assert_eq!(PdfObject::parse(&mut lexer).unwrap(), PdfObject::Real(3.14));
        assert_eq!(
            PdfObject::parse(&mut lexer).unwrap(),
            PdfObject::Name(PdfName("Name".to_string()))
        );
        assert_eq!(
            PdfObject::parse(&mut lexer).unwrap(),
            PdfObject::String(PdfString(b"Hello".to_vec()))
        );
    }

    #[test]
    fn test_parse_array() {
        // Test simple array without potential references
        let input = b"[100 200 300 /Name (test)]";
        let mut lexer = Lexer::new(Cursor::new(input));

        let obj = PdfObject::parse(&mut lexer).unwrap();
        let array = obj.as_array().unwrap();

        assert_eq!(array.len(), 5);
        assert_eq!(array.get(0).unwrap().as_integer(), Some(100));
        assert_eq!(array.get(1).unwrap().as_integer(), Some(200));
        assert_eq!(array.get(2).unwrap().as_integer(), Some(300));
        assert_eq!(array.get(3).unwrap().as_name().unwrap().as_str(), "Name");
        assert_eq!(
            array.get(4).unwrap().as_string().unwrap().as_bytes(),
            b"test"
        );
    }

    #[test]
    fn test_parse_array_with_references() {
        // Test array with references
        let input = b"[1 0 R 2 0 R]";
        let mut lexer = Lexer::new(Cursor::new(input));

        let obj = PdfObject::parse(&mut lexer).unwrap();
        let array = obj.as_array().unwrap();

        assert_eq!(array.len(), 2);
        assert!(array.get(0).unwrap().as_reference().is_some());
        assert!(array.get(1).unwrap().as_reference().is_some());
    }

    #[test]
    fn test_parse_dictionary() {
        let input = b"<< /Type /Page /Parent 1 0 R /MediaBox [0 0 612 792] >>";
        let mut lexer = Lexer::new(Cursor::new(input));

        let obj = PdfObject::parse(&mut lexer).unwrap();
        let dict = obj.as_dict().unwrap();

        assert_eq!(dict.get_type(), Some("Page"));
        assert!(dict.get("Parent").unwrap().as_reference().is_some());
        assert!(dict.get("MediaBox").unwrap().as_array().is_some());
    }
}
