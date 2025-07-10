//! Integration tests for the PDF parser

#[cfg(test)]
mod tests {
    use oxidize_pdf::parser::{header::*, lexer::*, objects::*, trailer::*, xref::*, PdfReader};
    use std::io::Cursor;

    /// Create a minimal valid PDF for testing
    fn create_test_pdf() -> Vec<u8> {
        let pdf = b"%PDF-1.4
%\xE2\xE3\xCF\xD3
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>
endobj
4 0 obj
<< /Title (Test PDF) /Author (oxidizePdf) /Creator (oxidizePdf test suite) >>
endobj
xref
0 5
0000000000 65535 f 
0000000015 00000 n 
0000000068 00000 n 
0000000125 00000 n 
0000000203 00000 n 
trailer
<< /Size 5 /Root 1 0 R /Info 4 0 R >>
startxref
306
%%EOF";
        pdf.to_vec()
    }

    #[test]
    fn test_lexer_tokenization() {
        let input = b"<< /Type /Page /Parent 2 0 R >>";
        let mut lexer = Lexer::new(Cursor::new(input));

        assert!(matches!(lexer.next_token().unwrap(), Token::DictStart));
        assert!(matches!(lexer.next_token().unwrap(), Token::Name(n) if n == "Type"));
        assert!(matches!(lexer.next_token().unwrap(), Token::Name(n) if n == "Page"));
        assert!(matches!(lexer.next_token().unwrap(), Token::Name(n) if n == "Parent"));
        assert!(matches!(
            lexer.next_token().unwrap(),
            Token::Reference(2, 0)
        ));
        assert!(matches!(lexer.next_token().unwrap(), Token::DictEnd));
    }

    #[test]
    fn test_object_parsing() {
        let input = b"<< /Type /Catalog /Pages 2 0 R >>";
        let mut lexer = Lexer::new(Cursor::new(input));

        let obj = PdfObject::parse(&mut lexer).unwrap();
        let dict = obj.as_dict().unwrap();

        assert_eq!(dict.get_type(), Some("Catalog"));
        assert!(matches!(
            dict.get("Pages"),
            Some(PdfObject::Reference(2, 0))
        ));
    }

    #[test]
    fn test_header_parsing() {
        let input = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n";
        let header = PdfHeader::parse(Cursor::new(input)).unwrap();

        assert_eq!(header.version.major, 1);
        assert_eq!(header.version.minor, 4);
        assert!(header.has_binary_marker);
    }

    #[test]
    fn test_parse_array_with_mixed_types() {
        let input = b"[0 0 612 792]";
        let mut lexer = Lexer::new(Cursor::new(input));

        let obj = PdfObject::parse(&mut lexer).unwrap();
        let array = obj.as_array().unwrap();

        assert_eq!(array.len(), 4);
        assert_eq!(array.get(0).unwrap().as_integer(), Some(0));
        assert_eq!(array.get(1).unwrap().as_integer(), Some(0));
        assert_eq!(array.get(2).unwrap().as_integer(), Some(612));
        assert_eq!(array.get(3).unwrap().as_integer(), Some(792));
    }

    #[test]
    fn test_parse_nested_structures() {
        let input = b"<< /Type /Page /Parent 2 0 R /Resources << /Font << /F1 4 0 R >> >> >>";
        let mut lexer = Lexer::new(Cursor::new(input));

        let obj = PdfObject::parse(&mut lexer).unwrap();
        let dict = obj.as_dict().unwrap();

        assert_eq!(dict.get_type(), Some("Page"));

        let resources = dict.get("Resources").unwrap().as_dict().unwrap();
        let font_dict = resources.get("Font").unwrap().as_dict().unwrap();
        assert!(matches!(
            font_dict.get("F1"),
            Some(PdfObject::Reference(4, 0))
        ));
    }

    #[test]
    fn test_string_parsing() {
        // Test literal string
        let input = b"(Hello World)";
        let mut lexer = Lexer::new(Cursor::new(input));
        let obj = PdfObject::parse(&mut lexer).unwrap();
        let string = obj.as_string().unwrap();
        assert_eq!(string.as_str().unwrap(), "Hello World");

        // Test hex string
        let input = b"<48656C6C6F>";
        let mut lexer = Lexer::new(Cursor::new(input));
        let obj = PdfObject::parse(&mut lexer).unwrap();
        let string = obj.as_string().unwrap();
        assert_eq!(string.as_str().unwrap(), "Hello");

        // Test string with escape sequences
        let input = b"(Line 1\\nLine 2)";
        let mut lexer = Lexer::new(Cursor::new(input));
        let obj = PdfObject::parse(&mut lexer).unwrap();
        let string = obj.as_string().unwrap();
        assert_eq!(string.as_str().unwrap(), "Line 1\nLine 2");
    }

    // Note: Full integration tests with PdfReader would require implementing Seek for Cursor
    // or using actual file I/O, which is beyond the scope of this test file
}
