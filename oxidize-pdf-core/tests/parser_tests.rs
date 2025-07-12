//! Integration tests for the PDF parser

#[cfg(test)]
mod tests {
    use oxidize_pdf::parser::{header::*, lexer::*, objects::*};
    use std::io::Cursor;


    #[test]
    fn test_lexer_tokenization() {
        let input = b"<< /Type /Page /Parent 2 0 R >>";
        let mut lexer = Lexer::new(Cursor::new(input));

        assert!(matches!(lexer.next_token().unwrap(), Token::DictStart));
        assert!(matches!(lexer.next_token().unwrap(), Token::Name(n) if n == "Type"));
        assert!(matches!(lexer.next_token().unwrap(), Token::Name(n) if n == "Page"));
        assert!(matches!(lexer.next_token().unwrap(), Token::Name(n) if n == "Parent"));
        // References are parsed as separate tokens by the lexer
        assert!(matches!(lexer.next_token().unwrap(), Token::Integer(2)));
        assert!(matches!(lexer.next_token().unwrap(), Token::Integer(0)));
        assert!(matches!(lexer.next_token().unwrap(), Token::Name(n) if n == "R"));
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
