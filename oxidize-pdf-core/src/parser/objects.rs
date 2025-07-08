//! PDF Object Parser
//! 
//! Parses PDF objects from tokens according to ISO 32000-1 Section 7.3

use super::{ParseError, ParseResult};
use super::lexer::{Lexer, Token};
use std::collections::HashMap;
use std::io::Read;

/// PDF Name object
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PdfName(pub String);

/// PDF String object  
#[derive(Debug, Clone, PartialEq)]
pub struct PdfString(pub Vec<u8>);

/// PDF Array object
#[derive(Debug, Clone, PartialEq)]
pub struct PdfArray(pub Vec<PdfObject>);

/// PDF Dictionary object
#[derive(Debug, Clone, PartialEq)]
pub struct PdfDictionary(pub HashMap<PdfName, PdfObject>);

/// PDF Stream object
#[derive(Debug, Clone, PartialEq)]
pub struct PdfStream {
    pub dict: PdfDictionary,
    pub data: Vec<u8>,
}

impl PdfStream {
    /// Get the decompressed stream data
    pub fn decode(&self) -> ParseResult<Vec<u8>> {
        super::filters::decode_stream(&self.data, &self.dict)
    }
    
    /// Get the raw (possibly compressed) stream data
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }
}

/// PDF Object types
#[derive(Debug, Clone, PartialEq)]
pub enum PdfObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(PdfString),
    Name(PdfName),
    Array(PdfArray),
    Dictionary(PdfDictionary),
    Stream(PdfStream),
    Reference(u32, u16), // object number, generation number
}

impl PdfObject {
    /// Parse a PDF object from a lexer
    pub fn parse<R: Read>(lexer: &mut Lexer<R>) -> ParseResult<Self> {
        let token = lexer.next_token()?;
        Self::parse_from_token(lexer, token)
    }
    
    /// Parse a PDF object starting from a specific token
    fn parse_from_token<R: Read>(lexer: &mut Lexer<R>, token: Token) -> ParseResult<Self> {
        match token {
            Token::Null => Ok(PdfObject::Null),
            Token::Boolean(b) => Ok(PdfObject::Boolean(b)),
            Token::Integer(i) => Ok(PdfObject::Integer(i)),
            Token::Real(r) => Ok(PdfObject::Real(r)),
            Token::String(s) => Ok(PdfObject::String(PdfString(s))),
            Token::Name(n) => Ok(PdfObject::Name(PdfName(n))),
            Token::Reference(obj, gen) => Ok(PdfObject::Reference(obj, gen)),
            Token::ArrayStart => Self::parse_array(lexer),
            Token::DictStart => Self::parse_dictionary_or_stream(lexer),
            Token::Comment(_) => {
                // Skip comments and parse next object
                Self::parse(lexer)
            }
            Token::Eof => Err(ParseError::SyntaxError {
                position: 0,
                message: "Unexpected end of file".to_string(),
            }),
            _ => Err(ParseError::UnexpectedToken {
                expected: "PDF object".to_string(),
                found: format!("{:?}", token),
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
                _ => {
                    // Not a stream, just a dictionary
                    // Push the token back for later processing
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
                        found: format!("{:?}", token),
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
        let length = dict.0.get(&PdfName("Length".to_string()))
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
                found: format!("{:?}", token),
            }),
        }
    }
    
    /// Check if this object is null
    pub fn is_null(&self) -> bool {
        matches!(self, PdfObject::Null)
    }
    
    /// Get as boolean
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
    
    /// Get as real number
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
    
    /// Get as reference
    pub fn as_reference(&self) -> Option<(u32, u16)> {
        match self {
            PdfObject::Reference(obj, gen) => Some((*obj, *gen)),
            _ => None,
        }
    }
}

impl PdfDictionary {
    /// Create a new empty dictionary
    pub fn new() -> Self {
        PdfDictionary(HashMap::new())
    }
    
    /// Get a value by key
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
    
    /// Get the dictionary type (value of /Type key)
    pub fn get_type(&self) -> Option<&str> {
        self.get("Type").and_then(|obj| obj.as_name()).map(|n| n.0.as_str())
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
    
    /// Get element at index
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
    
    /// Get as UTF-8 string if possible
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
        assert_eq!(PdfObject::parse(&mut lexer).unwrap(), PdfObject::Boolean(true));
        assert_eq!(PdfObject::parse(&mut lexer).unwrap(), PdfObject::Boolean(false));
        assert_eq!(PdfObject::parse(&mut lexer).unwrap(), PdfObject::Integer(123));
        assert_eq!(PdfObject::parse(&mut lexer).unwrap(), PdfObject::Integer(-456));
        assert_eq!(PdfObject::parse(&mut lexer).unwrap(), PdfObject::Real(3.14));
        assert_eq!(PdfObject::parse(&mut lexer).unwrap(), PdfObject::Name(PdfName("Name".to_string())));
        assert_eq!(PdfObject::parse(&mut lexer).unwrap(), PdfObject::String(PdfString(b"Hello".to_vec())));
    }
    
    #[test]
    fn test_parse_array() {
        let input = b"[1 2 3 /Name (test)]";
        let mut lexer = Lexer::new(Cursor::new(input));
        
        let obj = PdfObject::parse(&mut lexer).unwrap();
        let array = obj.as_array().unwrap();
        
        assert_eq!(array.len(), 5);
        assert_eq!(array.get(0).unwrap().as_integer(), Some(1));
        assert_eq!(array.get(1).unwrap().as_integer(), Some(2));
        assert_eq!(array.get(2).unwrap().as_integer(), Some(3));
        assert_eq!(array.get(3).unwrap().as_name().unwrap().as_str(), "Name");
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