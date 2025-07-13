//! PDF Lexer
//!
//! Tokenizes PDF syntax according to ISO 32000-1 Section 7.2

use super::{ParseError, ParseResult};
use std::io::Read;

/// PDF Token types
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Boolean: true or false
    Boolean(bool),

    /// Integer number
    Integer(i64),

    /// Real number
    Real(f64),

    /// String (literal or hexadecimal)
    String(Vec<u8>),

    /// Name object (e.g., /Type)
    Name(String),

    /// Left square bracket [
    ArrayStart,

    /// Right square bracket ]
    ArrayEnd,

    /// Dictionary start <<
    DictStart,

    /// Dictionary end >>
    DictEnd,

    /// Stream keyword
    Stream,

    /// Endstream keyword
    EndStream,

    /// Obj keyword
    Obj,

    /// Endobj keyword
    EndObj,

    /// StartXRef keyword
    StartXRef,

    /// Reference (e.g., 1 0 R)
    Reference(u32, u16),

    /// Null object
    Null,

    /// Comment (usually ignored)
    Comment(String),

    /// End of file
    Eof,
}

/// PDF Lexer for tokenizing PDF content
pub struct Lexer<R: Read> {
    reader: std::io::BufReader<R>,
    #[allow(dead_code)]
    buffer: Vec<u8>,
    position: usize,
    peek_buffer: Option<u8>,
    token_buffer: Vec<Token>,
}

impl<R: Read> Lexer<R> {
    /// Create a new lexer from a reader
    pub fn new(reader: R) -> Self {
        Self {
            reader: std::io::BufReader::new(reader),
            buffer: Vec::with_capacity(1024),
            position: 0,
            peek_buffer: None,
            token_buffer: Vec::new(),
        }
    }

    /// Get the next token
    pub fn next_token(&mut self) -> ParseResult<Token> {
        // Check if we have a pushed-back token
        if let Some(token) = self.token_buffer.pop() {
            return Ok(token);
        }

        self.skip_whitespace()?;

        let ch = match self.peek_char()? {
            Some(ch) => ch,
            None => return Ok(Token::Eof),
        };

        match ch {
            b'%' => self.read_comment(),
            b'/' => self.read_name(),
            b'(' => self.read_literal_string(),
            b'<' => self.read_angle_bracket(),
            b'>' => {
                self.consume_char()?;
                if self.peek_char()? == Some(b'>') {
                    self.consume_char()?;
                    Ok(Token::DictEnd)
                } else {
                    Err(ParseError::SyntaxError {
                        position: self.position,
                        message: "Expected '>' after '>'".to_string(),
                    })
                }
            }
            b'[' => {
                self.consume_char()?;
                Ok(Token::ArrayStart)
            }
            b']' => {
                self.consume_char()?;
                Ok(Token::ArrayEnd)
            }
            b't' | b'f' => self.read_boolean(),
            b'n' => self.read_null(),
            b'+' | b'-' | b'0'..=b'9' | b'.' => self.read_number(),
            b'R' => {
                // R could be a keyword (for references)
                self.consume_char()?;
                Ok(Token::Name("R".to_string()))
            }
            _ if ch.is_ascii_alphabetic() => self.read_keyword(),
            _ => Err(ParseError::SyntaxError {
                position: self.position,
                message: format!("Unexpected character: {}", ch as char),
            }),
        }
    }

    /// Peek at the next character without consuming it
    fn peek_char(&mut self) -> ParseResult<Option<u8>> {
        if let Some(ch) = self.peek_buffer {
            return Ok(Some(ch));
        }

        let mut buf = [0u8; 1];
        match self.reader.read_exact(&mut buf) {
            Ok(_) => {
                self.peek_buffer = Some(buf[0]);
                Ok(Some(buf[0]))
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Consume the next character
    fn consume_char(&mut self) -> ParseResult<Option<u8>> {
        let ch = self.peek_char()?;
        if ch.is_some() {
            self.peek_buffer = None;
            self.position += 1;
        }
        Ok(ch)
    }

    /// Skip whitespace and return the number of bytes skipped
    pub(crate) fn skip_whitespace(&mut self) -> ParseResult<usize> {
        let mut count = 0;
        while let Some(ch) = self.peek_char()? {
            if ch.is_ascii_whitespace() {
                self.consume_char()?;
                count += 1;
            } else {
                break;
            }
        }
        Ok(count)
    }

    /// Read a comment (from % to end of line)
    fn read_comment(&mut self) -> ParseResult<Token> {
        self.consume_char()?; // consume '%'
        let mut comment = String::new();

        while let Some(ch) = self.peek_char()? {
            if ch == b'\n' || ch == b'\r' {
                break;
            }
            self.consume_char()?;
            comment.push(ch as char);
        }

        Ok(Token::Comment(comment))
    }

    /// Read a name object (e.g., /Type)
    fn read_name(&mut self) -> ParseResult<Token> {
        self.consume_char()?; // consume '/'
        let mut name = String::new();

        while let Some(ch) = self.peek_char()? {
            if ch.is_ascii_whitespace()
                || matches!(ch, b'/' | b'<' | b'>' | b'[' | b']' | b'(' | b')' | b'%')
            {
                break;
            }
            self.consume_char()?;

            // Handle hex codes in names (e.g., /A#20B means /A B)
            if ch == b'#' {
                let hex1 = self
                    .consume_char()?
                    .ok_or_else(|| ParseError::SyntaxError {
                        position: self.position,
                        message: "Incomplete hex code in name".to_string(),
                    })?;
                let hex2 = self
                    .consume_char()?
                    .ok_or_else(|| ParseError::SyntaxError {
                        position: self.position,
                        message: "Incomplete hex code in name".to_string(),
                    })?;

                let value = u8::from_str_radix(&format!("{}{}", hex1 as char, hex2 as char), 16)
                    .map_err(|_| ParseError::SyntaxError {
                        position: self.position,
                        message: "Invalid hex code in name".to_string(),
                    })?;

                name.push(value as char);
            } else {
                name.push(ch as char);
            }
        }

        Ok(Token::Name(name))
    }

    /// Read a literal string (parentheses)
    fn read_literal_string(&mut self) -> ParseResult<Token> {
        self.consume_char()?; // consume '('
        let mut string = Vec::new();
        let mut paren_depth = 1;
        let mut escape = false;

        while paren_depth > 0 {
            let ch = self
                .consume_char()?
                .ok_or_else(|| ParseError::SyntaxError {
                    position: self.position,
                    message: "Unterminated string".to_string(),
                })?;

            if escape {
                let escaped = match ch {
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b't' => b'\t',
                    b'b' => b'\x08',
                    b'f' => b'\x0C',
                    b'(' => b'(',
                    b')' => b')',
                    b'\\' => b'\\',
                    b'0'..=b'7' => {
                        // Octal escape sequence
                        let mut value = ch - b'0';
                        for _ in 0..2 {
                            if let Some(next) = self.peek_char()? {
                                if matches!(next, b'0'..=b'7') {
                                    self.consume_char()?;
                                    value = value * 8 + (next - b'0');
                                } else {
                                    break;
                                }
                            }
                        }
                        value
                    }
                    _ => ch, // Unknown escape, use literal
                };
                string.push(escaped);
                escape = false;
            } else {
                match ch {
                    b'\\' => escape = true,
                    b'(' => {
                        string.push(ch);
                        paren_depth += 1;
                    }
                    b')' => {
                        paren_depth -= 1;
                        if paren_depth > 0 {
                            string.push(ch);
                        }
                    }
                    _ => string.push(ch),
                }
            }
        }

        Ok(Token::String(string))
    }

    /// Read angle bracket tokens (hex strings or dict markers)
    fn read_angle_bracket(&mut self) -> ParseResult<Token> {
        self.consume_char()?; // consume '<'

        if self.peek_char()? == Some(b'<') {
            self.consume_char()?;
            Ok(Token::DictStart)
        } else {
            // Hex string
            let mut hex_chars = String::new();
            let mut found_end = false;

            while let Some(ch) = self.peek_char()? {
                if ch == b'>' {
                    self.consume_char()?;
                    found_end = true;
                    break;
                }
                self.consume_char()?;
                if ch.is_ascii_hexdigit() {
                    hex_chars.push(ch as char);
                } else if !ch.is_ascii_whitespace() {
                    return Err(ParseError::SyntaxError {
                        position: self.position,
                        message: "Invalid character in hex string".to_string(),
                    });
                }
            }

            if !found_end {
                return Err(ParseError::SyntaxError {
                    position: self.position,
                    message: "Unterminated hex string".to_string(),
                });
            }

            // Pad with 0 if odd number of digits
            if hex_chars.len() % 2 != 0 {
                hex_chars.push('0');
            }

            // Convert hex to bytes
            let mut bytes = Vec::new();
            for chunk in hex_chars.as_bytes().chunks(2) {
                let hex_str = std::str::from_utf8(chunk).unwrap();
                let byte =
                    u8::from_str_radix(hex_str, 16).map_err(|_| ParseError::SyntaxError {
                        position: self.position,
                        message: "Invalid hex string".to_string(),
                    })?;
                bytes.push(byte);
            }

            Ok(Token::String(bytes))
        }
    }

    /// Read boolean (true/false)
    fn read_boolean(&mut self) -> ParseResult<Token> {
        let word = self.read_word()?;
        match word.as_str() {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            _ => {
                // Not a boolean, might be a keyword
                self.process_keyword(word)
            }
        }
    }

    /// Read null
    fn read_null(&mut self) -> ParseResult<Token> {
        let word = self.read_word()?;
        if word == "null" {
            Ok(Token::Null)
        } else {
            // Not null, might be a keyword
            self.process_keyword(word)
        }
    }

    /// Read a number (integer or real)
    fn read_number(&mut self) -> ParseResult<Token> {
        let mut number_str = String::new();
        let mut has_dot = false;

        // Handle sign - consume it first
        if let Some(ch) = self.peek_char()? {
            if ch == b'+' || ch == b'-' {
                self.consume_char()?;
                number_str.push(ch as char);

                // After sign, we must have at least one digit
                if let Some(next) = self.peek_char()? {
                    if !next.is_ascii_digit() && next != b'.' {
                        return Err(ParseError::SyntaxError {
                            position: self.position,
                            message: "Expected digit after sign".to_string(),
                        });
                    }
                }
            }
        }

        // Read digits and decimal point
        while let Some(ch) = self.peek_char()? {
            match ch {
                b'0'..=b'9' => {
                    self.consume_char()?;
                    number_str.push(ch as char);
                }
                b'.' if !has_dot => {
                    self.consume_char()?;
                    number_str.push(ch as char);
                    has_dot = true;
                }
                _ => break,
            }
        }

        // Don't try to parse references here - let the parser handle it
        // References are just "num num R" and can be handled at a higher level

        // Parse as number
        if has_dot {
            let value = number_str
                .parse::<f64>()
                .map_err(|_| ParseError::SyntaxError {
                    position: self.position,
                    message: format!("Invalid real number: '{number_str}'"),
                })?;
            Ok(Token::Real(value))
        } else {
            let value = number_str
                .parse::<i64>()
                .map_err(|_| ParseError::SyntaxError {
                    position: self.position,
                    message: format!("Invalid integer: '{number_str}'"),
                })?;
            Ok(Token::Integer(value))
        }
    }

    /// Read a keyword
    fn read_keyword(&mut self) -> ParseResult<Token> {
        let word = self.read_word()?;
        self.process_keyword(word)
    }

    /// Process a word as a keyword
    fn process_keyword(&self, word: String) -> ParseResult<Token> {
        match word.as_str() {
            "stream" => Ok(Token::Stream),
            "endstream" => Ok(Token::EndStream),
            "obj" => Ok(Token::Obj),
            "endobj" => Ok(Token::EndObj),
            "startxref" => Ok(Token::StartXRef),
            _ => Err(ParseError::SyntaxError {
                position: self.position,
                message: format!("Unknown keyword: {word}"),
            }),
        }
    }

    /// Read a word (sequence of non-delimiter characters)
    fn read_word(&mut self) -> ParseResult<String> {
        let mut word = String::new();

        while let Some(ch) = self.peek_char()? {
            if ch.is_ascii_whitespace()
                || matches!(ch, b'/' | b'<' | b'>' | b'[' | b']' | b'(' | b')' | b'%')
            {
                break;
            }
            self.consume_char()?;
            word.push(ch as char);
        }

        Ok(word)
    }

    /// Read a sequence of digits
    #[allow(dead_code)]
    fn read_digits(&mut self) -> ParseResult<String> {
        let mut digits = String::new();

        while let Some(ch) = self.peek_char()? {
            if ch.is_ascii_digit() {
                self.consume_char()?;
                digits.push(ch as char);
            } else {
                break;
            }
        }

        Ok(digits)
    }

    /// Read a newline sequence (CR, LF, or CRLF)
    pub fn read_newline(&mut self) -> ParseResult<()> {
        match self.peek_char()? {
            Some(b'\r') => {
                self.consume_char()?;
                // Check for CRLF
                if self.peek_char()? == Some(b'\n') {
                    self.consume_char()?;
                }
                Ok(())
            }
            Some(b'\n') => {
                self.consume_char()?;
                Ok(())
            }
            _ => Err(ParseError::SyntaxError {
                position: self.position,
                message: "Expected newline".to_string(),
            }),
        }
    }

    /// Read exactly n bytes
    pub fn read_bytes(&mut self, n: usize) -> ParseResult<Vec<u8>> {
        let mut bytes = vec![0u8; n];
        self.reader.read_exact(&mut bytes)?;
        self.position += n;
        Ok(bytes)
    }

    /// Read until a specific byte sequence is found
    pub fn read_until_sequence(&mut self, sequence: &[u8]) -> ParseResult<Vec<u8>> {
        let mut result = Vec::new();
        let mut match_pos = 0;

        while let Some(ch) = self.consume_char()? {
            result.push(ch);

            if ch == sequence[match_pos] {
                match_pos += 1;
                if match_pos == sequence.len() {
                    // Found the sequence, remove it from result
                    result.truncate(result.len() - sequence.len());
                    break;
                }
            } else if ch == sequence[0] {
                match_pos = 1;
            } else {
                match_pos = 0;
            }
        }

        if match_pos < sequence.len() {
            return Err(ParseError::SyntaxError {
                position: self.position,
                message: format!("Sequence {sequence:?} not found"),
            });
        }

        Ok(result)
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Push back a token to be returned by the next call to next_token
    pub fn push_token(&mut self, token: Token) {
        self.token_buffer.push(token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_lexer_basic_tokens() {
        // Test positive and negative numbers
        let input = b"123 -456 3.14 true false null /Name";
        let mut lexer = Lexer::new(Cursor::new(input));

        assert_eq!(lexer.next_token().unwrap(), Token::Integer(123));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(-456));
        assert_eq!(lexer.next_token().unwrap(), Token::Real(3.14));
        assert_eq!(lexer.next_token().unwrap(), Token::Boolean(true));
        assert_eq!(lexer.next_token().unwrap(), Token::Boolean(false));
        assert_eq!(lexer.next_token().unwrap(), Token::Null);
        assert_eq!(lexer.next_token().unwrap(), Token::Name("Name".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_lexer_negative_numbers() {
        // Test negative numbers without space
        let input = b"-123 -45.67";
        let mut lexer = Lexer::new(Cursor::new(input));

        assert_eq!(lexer.next_token().unwrap(), Token::Integer(-123));
        assert_eq!(lexer.next_token().unwrap(), Token::Real(-45.67));
    }

    #[test]
    fn test_lexer_strings() {
        let input = b"(Hello World) <48656C6C6F>";
        let mut lexer = Lexer::new(Cursor::new(input));

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String(b"Hello World".to_vec())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String(b"Hello".to_vec())
        );
    }

    #[test]
    fn test_lexer_dictionaries() {
        let input = b"<< /Type /Page >>";
        let mut lexer = Lexer::new(Cursor::new(input));

        assert_eq!(lexer.next_token().unwrap(), Token::DictStart);
        assert_eq!(lexer.next_token().unwrap(), Token::Name("Type".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::Name("Page".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::DictEnd);
    }

    #[test]
    fn test_lexer_arrays() {
        let input = b"[1 2 3]";
        let mut lexer = Lexer::new(Cursor::new(input));

        assert_eq!(lexer.next_token().unwrap(), Token::ArrayStart);
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(1));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(2));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(3));
        assert_eq!(lexer.next_token().unwrap(), Token::ArrayEnd);
    }

    #[test]
    fn test_lexer_references() {
        let input = b"1 0 R 25 1 R";
        let mut lexer = Lexer::new(Cursor::new(input));

        // Now references are parsed as separate tokens
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(1));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(0));
        // 'R' should be parsed as a keyword or name
        match lexer.next_token().unwrap() {
            Token::Name(s) if s == "R" => {} // Could be a name
            other => panic!("Expected R token, got {other:?}"),
        }

        assert_eq!(lexer.next_token().unwrap(), Token::Integer(25));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(1));
        match lexer.next_token().unwrap() {
            Token::Name(s) if s == "R" => {} // Could be a name
            other => panic!("Expected R token, got {other:?}"),
        }
    }

    #[test]
    fn test_lexer_comments() {
        let input = b"%PDF-1.7\n123";
        let mut lexer = Lexer::new(Cursor::new(input));

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Comment("PDF-1.7".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(123));
    }
}
