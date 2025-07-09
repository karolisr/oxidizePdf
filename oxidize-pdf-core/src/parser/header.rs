//! PDF Header Parser
//! 
//! Parses PDF header and version according to ISO 32000-1 Section 7.5.2

use super::{ParseError, ParseResult};
use std::io::{Read, BufRead, BufReader};

/// PDF Version information
#[derive(Debug, Clone, PartialEq)]
pub struct PdfVersion {
    pub major: u8,
    pub minor: u8,
}

impl PdfVersion {
    /// Create a new PDF version
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }
    
    /// Check if this version is supported
    pub fn is_supported(&self) -> bool {
        // We support PDF 1.0 through 2.0
        match (self.major, self.minor) {
            (1, 0..=7) => true,
            (2, 0) => true,
            _ => false,
        }
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

/// PDF Header information
#[derive(Debug, Clone)]
pub struct PdfHeader {
    pub version: PdfVersion,
    pub has_binary_marker: bool,
}

impl PdfHeader {
    /// Parse PDF header from a reader
    pub fn parse<R: Read>(reader: R) -> ParseResult<Self> {
        let mut buf_reader = BufReader::new(reader);
        let mut header = Self::parse_version_line(&mut buf_reader)?;
        
        // Check for binary marker (recommended for PDF 1.2+)
        header.has_binary_marker = Self::check_binary_marker(&mut buf_reader)?;
        
        Ok(header)
    }
    
    /// Parse the PDF version line
    fn parse_version_line<R: BufRead>(reader: &mut R) -> ParseResult<Self> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        
        // Remove newline characters
        let line = line.trim_end();
        
        // PDF header must start with %PDF-
        if !line.starts_with("%PDF-") {
            return Err(ParseError::InvalidHeader);
        }
        
        // Extract version
        let version_str = &line[5..];
        let parts: Vec<&str> = version_str.split('.').collect();
        
        if parts.len() != 2 {
            return Err(ParseError::InvalidHeader);
        }
        
        let major = parts[0].parse::<u8>().map_err(|_| ParseError::InvalidHeader)?;
        let minor = parts[1].parse::<u8>().map_err(|_| ParseError::InvalidHeader)?;
        
        let version = PdfVersion::new(major, minor);
        
        if !version.is_supported() {
            return Err(ParseError::UnsupportedVersion(version.to_string()));
        }
        
        Ok(PdfHeader {
            version,
            has_binary_marker: false,
        })
    }
    
    /// Check for binary marker comment
    fn check_binary_marker<R: BufRead>(reader: &mut R) -> ParseResult<bool> {
        let mut buffer = Vec::new();
        
        // Read bytes until we find a newline or EOF
        loop {
            let mut byte = [0u8; 1];
            match reader.read_exact(&mut byte) {
                Ok(_) => {
                    buffer.push(byte[0]);
                    if byte[0] == b'\n' || byte[0] == b'\r' {
                        break;
                    }
                    // Limit line length to prevent excessive memory usage
                    if buffer.len() > 1024 {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }
        
        if buffer.is_empty() {
            return Ok(false);
        }
        
        // Binary marker should be a comment with at least 4 binary characters
        if buffer.first() == Some(&b'%') {
            let binary_count = buffer.iter()
                .skip(1) // Skip the %
                .filter(|&&b| b >= 128)
                .count();
            
            Ok(binary_count >= 4)
        } else {
            // Not a comment, probably start of document content
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_parse_pdf_header_basic() {
        let input = b"%PDF-1.7\n";
        let header = PdfHeader::parse(Cursor::new(input)).unwrap();
        
        assert_eq!(header.version.major, 1);
        assert_eq!(header.version.minor, 7);
        assert!(!header.has_binary_marker);
    }
    
    #[test]
    fn test_parse_pdf_header_with_binary_marker() {
        let input = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n";
        let header = PdfHeader::parse(Cursor::new(input)).unwrap();
        
        assert_eq!(header.version.major, 1);
        assert_eq!(header.version.minor, 4);
        assert!(header.has_binary_marker);
    }
    
    #[test]
    fn test_parse_pdf_20() {
        let input = b"%PDF-2.0\n";
        let header = PdfHeader::parse(Cursor::new(input)).unwrap();
        
        assert_eq!(header.version.major, 2);
        assert_eq!(header.version.minor, 0);
    }
    
    #[test]
    fn test_invalid_header() {
        let input = b"Not a PDF\n";
        let result = PdfHeader::parse(Cursor::new(input));
        
        assert!(matches!(result, Err(ParseError::InvalidHeader)));
    }
    
    #[test]
    fn test_unsupported_version() {
        let input = b"%PDF-3.0\n";
        let result = PdfHeader::parse(Cursor::new(input));
        
        assert!(matches!(result, Err(ParseError::UnsupportedVersion(_))));
    }
}