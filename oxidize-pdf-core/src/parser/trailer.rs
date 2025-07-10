//! PDF Trailer Parser
//!
//! Parses PDF trailer according to ISO 32000-1 Section 7.5.5

use super::objects::{PdfDictionary, PdfObject};
use super::{ParseError, ParseResult};

/// PDF Trailer information
#[derive(Debug, Clone)]
pub struct PdfTrailer {
    /// The trailer dictionary
    pub dict: PdfDictionary,
    /// Byte offset of previous xref section (if any)
    pub prev: Option<u64>,
    /// Byte offset of this xref section
    pub xref_offset: u64,
}

impl PdfTrailer {
    /// Parse trailer from a dictionary
    pub fn from_dict(dict: PdfDictionary, xref_offset: u64) -> ParseResult<Self> {
        // Extract previous xref offset if present
        let prev = dict
            .get("Prev")
            .and_then(|obj| obj.as_integer())
            .map(|i| i as u64);

        Ok(PdfTrailer {
            dict,
            prev,
            xref_offset,
        })
    }

    /// Get the size (number of entries in xref table)
    pub fn size(&self) -> ParseResult<u32> {
        self.dict
            .get("Size")
            .and_then(|obj| obj.as_integer())
            .map(|i| i as u32)
            .ok_or_else(|| ParseError::MissingKey("Size".to_string()))
    }

    /// Get the root object reference (document catalog)
    pub fn root(&self) -> ParseResult<(u32, u16)> {
        self.dict
            .get("Root")
            .and_then(|obj| obj.as_reference())
            .ok_or_else(|| ParseError::MissingKey("Root".to_string()))
    }

    /// Get the info object reference (document information dictionary)
    pub fn info(&self) -> Option<(u32, u16)> {
        self.dict.get("Info").and_then(|obj| obj.as_reference())
    }

    /// Get the ID array (file identifiers)
    pub fn id(&self) -> Option<&PdfObject> {
        self.dict.get("ID")
    }

    /// Check if this PDF is encrypted
    pub fn is_encrypted(&self) -> bool {
        self.dict.contains_key("Encrypt")
    }

    /// Get the encryption dictionary reference
    pub fn encrypt(&self) -> Option<(u32, u16)> {
        self.dict.get("Encrypt").and_then(|obj| obj.as_reference())
    }

    /// Validate the trailer dictionary
    pub fn validate(&self) -> ParseResult<()> {
        // Required entries
        self.size()?;
        self.root()?;

        // If encrypted, we currently don't support it
        if self.is_encrypted() {
            return Err(ParseError::EncryptionNotSupported);
        }

        Ok(())
    }
}

/// Represents the complete trailer chain for PDFs with updates
#[derive(Debug)]
pub struct TrailerChain {
    /// List of trailers from newest to oldest
    trailers: Vec<PdfTrailer>,
}

impl TrailerChain {
    /// Create a new trailer chain with a single trailer
    pub fn new(trailer: PdfTrailer) -> Self {
        Self {
            trailers: vec![trailer],
        }
    }

    /// Add an older trailer to the chain
    pub fn add_previous(&mut self, trailer: PdfTrailer) {
        self.trailers.push(trailer);
    }

    /// Get the most recent trailer
    pub fn current(&self) -> &PdfTrailer {
        &self.trailers[0]
    }

    /// Get all trailers in the chain
    pub fn all(&self) -> &[PdfTrailer] {
        &self.trailers
    }

    /// Check if there are previous versions
    pub fn has_previous(&self) -> bool {
        self.trailers.len() > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_trailer_basic() {
        let mut dict = PdfDictionary::new();
        dict.insert("Size".to_string(), PdfObject::Integer(100));
        dict.insert("Root".to_string(), PdfObject::Reference(1, 0));

        let trailer = PdfTrailer::from_dict(dict, 12345).unwrap();

        assert_eq!(trailer.size().unwrap(), 100);
        assert_eq!(trailer.root().unwrap(), (1, 0));
        assert!(trailer.info().is_none());
        assert!(!trailer.is_encrypted());
    }

    #[test]
    fn test_trailer_with_prev() {
        let mut dict = PdfDictionary::new();
        dict.insert("Size".to_string(), PdfObject::Integer(200));
        dict.insert("Root".to_string(), PdfObject::Reference(1, 0));
        dict.insert("Prev".to_string(), PdfObject::Integer(5000));

        let trailer = PdfTrailer::from_dict(dict, 20000).unwrap();

        assert_eq!(trailer.prev, Some(5000));
        assert_eq!(trailer.xref_offset, 20000);
    }

    #[test]
    fn test_trailer_validation() {
        // Missing Size
        let mut dict = PdfDictionary::new();
        dict.insert("Root".to_string(), PdfObject::Reference(1, 0));

        let trailer = PdfTrailer::from_dict(dict, 12345).unwrap();
        assert!(trailer.validate().is_err());

        // Missing Root
        let mut dict = PdfDictionary::new();
        dict.insert("Size".to_string(), PdfObject::Integer(100));

        let trailer = PdfTrailer::from_dict(dict, 12345).unwrap();
        assert!(trailer.validate().is_err());

        // Encrypted
        let mut dict = PdfDictionary::new();
        dict.insert("Size".to_string(), PdfObject::Integer(100));
        dict.insert("Root".to_string(), PdfObject::Reference(1, 0));
        dict.insert("Encrypt".to_string(), PdfObject::Reference(10, 0));

        let trailer = PdfTrailer::from_dict(dict, 12345).unwrap();
        assert!(matches!(
            trailer.validate(),
            Err(ParseError::EncryptionNotSupported)
        ));
    }
}
