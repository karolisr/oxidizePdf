//! PDF Content Stream Parser
//!
//! This module implements parsing of PDF content streams according to the PDF specification.
//! Content streams contain the actual drawing instructions that render text, graphics, and images
//! on PDF pages.

use super::{ParseError, ParseResult};
use std::collections::HashMap;

/// Represents a single operator in a PDF content stream
#[derive(Debug, Clone, PartialEq)]
pub enum ContentOperation {
    // Text object operators
    BeginText, // BT
    EndText,   // ET

    // Text state operators
    SetCharSpacing(f32),       // Tc
    SetWordSpacing(f32),       // Tw
    SetHorizontalScaling(f32), // Tz
    SetLeading(f32),           // TL
    SetFont(String, f32),      // Tf
    SetTextRenderMode(i32),    // Tr
    SetTextRise(f32),          // Ts

    // Text positioning operators
    MoveText(f32, f32),                          // Td
    MoveTextSetLeading(f32, f32),                // TD
    SetTextMatrix(f32, f32, f32, f32, f32, f32), // Tm
    NextLine,                                    // T*

    // Text showing operators
    ShowText(Vec<u8>),                             // Tj
    ShowTextArray(Vec<TextElement>),               // TJ
    NextLineShowText(Vec<u8>),                     // '
    SetSpacingNextLineShowText(f32, f32, Vec<u8>), // "

    // Graphics state operators
    SaveGraphicsState,                                // q
    RestoreGraphicsState,                             // Q
    SetTransformMatrix(f32, f32, f32, f32, f32, f32), // cm
    SetLineWidth(f32),                                // w
    SetLineCap(i32),                                  // J
    SetLineJoin(i32),                                 // j
    SetMiterLimit(f32),                               // M
    SetDashPattern(Vec<f32>, f32),                    // d
    SetIntent(String),                                // ri
    SetFlatness(f32),                                 // i
    SetGraphicsStateParams(String),                   // gs

    // Path construction operators
    MoveTo(f32, f32),                      // m
    LineTo(f32, f32),                      // l
    CurveTo(f32, f32, f32, f32, f32, f32), // c
    CurveToV(f32, f32, f32, f32),          // v
    CurveToY(f32, f32, f32, f32),          // y
    ClosePath,                             // h
    Rectangle(f32, f32, f32, f32),         // re

    // Path painting operators
    Stroke,                 // S
    CloseStroke,            // s
    Fill,                   // f or F
    FillEvenOdd,            // f*
    FillStroke,             // B
    FillStrokeEvenOdd,      // B*
    CloseFillStroke,        // b
    CloseFillStrokeEvenOdd, // b*
    EndPath,                // n

    // Clipping path operators
    Clip,        // W
    ClipEvenOdd, // W*

    // Color operators
    SetStrokingColorSpace(String),          // CS
    SetNonStrokingColorSpace(String),       // cs
    SetStrokingColor(Vec<f32>),             // SC, SCN
    SetNonStrokingColor(Vec<f32>),          // sc, scn
    SetStrokingGray(f32),                   // G
    SetNonStrokingGray(f32),                // g
    SetStrokingRGB(f32, f32, f32),          // RG
    SetNonStrokingRGB(f32, f32, f32),       // rg
    SetStrokingCMYK(f32, f32, f32, f32),    // K
    SetNonStrokingCMYK(f32, f32, f32, f32), // k

    // Shading operators
    ShadingFill(String), // sh

    // Inline image operators
    BeginInlineImage,         // BI
    InlineImageData(Vec<u8>), // ID...EI

    // XObject operators
    PaintXObject(String), // Do

    // Marked content operators
    BeginMarkedContent(String),                                   // BMC
    BeginMarkedContentWithProps(String, HashMap<String, String>), // BDC
    EndMarkedContent,                                             // EMC
    DefineMarkedContentPoint(String),                             // MP
    DefineMarkedContentPointWithProps(String, HashMap<String, String>), // DP

    // Compatibility operators
    BeginCompatibility, // BX
    EndCompatibility,   // EX
}

/// Represents a text element in a TJ array
#[derive(Debug, Clone, PartialEq)]
pub enum TextElement {
    Text(Vec<u8>),
    Spacing(f32),
}

/// Token types in content streams
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Token {
    Number(f32),
    Integer(i32),
    String(Vec<u8>),
    HexString(Vec<u8>),
    Name(String),
    Operator(String),
    ArrayStart,
    ArrayEnd,
    DictStart,
    DictEnd,
}

/// Content stream tokenizer
pub struct ContentTokenizer<'a> {
    input: &'a [u8],
    position: usize,
}

impl<'a> ContentTokenizer<'a> {
    /// Create a new tokenizer for the given input
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, position: 0 }
    }

    /// Get the next token from the stream
    pub(super) fn next_token(&mut self) -> ParseResult<Option<Token>> {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Ok(None);
        }

        let ch = self.input[self.position];

        match ch {
            // Numbers
            b'+' | b'-' | b'.' | b'0'..=b'9' => self.read_number(),

            // Strings
            b'(' => self.read_literal_string(),
            b'<' => {
                if self.peek_next() == Some(b'<') {
                    self.position += 2;
                    Ok(Some(Token::DictStart))
                } else {
                    self.read_hex_string()
                }
            }
            b'>' => {
                if self.peek_next() == Some(b'>') {
                    self.position += 2;
                    Ok(Some(Token::DictEnd))
                } else {
                    Err(ParseError::SyntaxError {
                        position: self.position,
                        message: "Unexpected '>'".to_string(),
                    })
                }
            }

            // Arrays
            b'[' => {
                self.position += 1;
                Ok(Some(Token::ArrayStart))
            }
            b']' => {
                self.position += 1;
                Ok(Some(Token::ArrayEnd))
            }

            // Names
            b'/' => self.read_name(),

            // Operators or other tokens
            _ => self.read_operator(),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() {
            match self.input[self.position] {
                b' ' | b'\t' | b'\r' | b'\n' | b'\x0C' => self.position += 1,
                b'%' => self.skip_comment(),
                _ => break,
            }
        }
    }

    fn skip_comment(&mut self) {
        while self.position < self.input.len() && self.input[self.position] != b'\n' {
            self.position += 1;
        }
    }

    fn peek_next(&self) -> Option<u8> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }

    fn read_number(&mut self) -> ParseResult<Option<Token>> {
        let start = self.position;
        let mut has_dot = false;

        // Handle optional sign
        if self.position < self.input.len()
            && (self.input[self.position] == b'+' || self.input[self.position] == b'-')
        {
            self.position += 1;
        }

        // Read digits and optional decimal point
        while self.position < self.input.len() {
            match self.input[self.position] {
                b'0'..=b'9' => self.position += 1,
                b'.' if !has_dot => {
                    has_dot = true;
                    self.position += 1;
                }
                _ => break,
            }
        }

        let num_str = std::str::from_utf8(&self.input[start..self.position]).map_err(|_| {
            ParseError::SyntaxError {
                position: start,
                message: "Invalid number format".to_string(),
            }
        })?;

        if has_dot {
            let value = num_str
                .parse::<f32>()
                .map_err(|_| ParseError::SyntaxError {
                    position: start,
                    message: "Invalid float number".to_string(),
                })?;
            Ok(Some(Token::Number(value)))
        } else {
            let value = num_str
                .parse::<i32>()
                .map_err(|_| ParseError::SyntaxError {
                    position: start,
                    message: "Invalid integer number".to_string(),
                })?;
            Ok(Some(Token::Integer(value)))
        }
    }

    fn read_literal_string(&mut self) -> ParseResult<Option<Token>> {
        self.position += 1; // Skip opening '('
        let mut result = Vec::new();
        let mut paren_depth = 1;
        let mut escape = false;

        while self.position < self.input.len() && paren_depth > 0 {
            let ch = self.input[self.position];
            self.position += 1;

            if escape {
                match ch {
                    b'n' => result.push(b'\n'),
                    b'r' => result.push(b'\r'),
                    b't' => result.push(b'\t'),
                    b'b' => result.push(b'\x08'),
                    b'f' => result.push(b'\x0C'),
                    b'(' => result.push(b'('),
                    b')' => result.push(b')'),
                    b'\\' => result.push(b'\\'),
                    b'0'..=b'7' => {
                        // Octal escape sequence
                        self.position -= 1;
                        let octal_value = self.read_octal_escape()?;
                        result.push(octal_value);
                    }
                    _ => result.push(ch), // Unknown escape, treat as literal
                }
                escape = false;
            } else {
                match ch {
                    b'\\' => escape = true,
                    b'(' => {
                        paren_depth += 1;
                        result.push(ch);
                    }
                    b')' => {
                        paren_depth -= 1;
                        if paren_depth > 0 {
                            result.push(ch);
                        }
                    }
                    _ => result.push(ch),
                }
            }
        }

        Ok(Some(Token::String(result)))
    }

    fn read_octal_escape(&mut self) -> ParseResult<u8> {
        let mut value = 0u8;
        let mut count = 0;

        while count < 3 && self.position < self.input.len() {
            match self.input[self.position] {
                b'0'..=b'7' => {
                    value = value * 8 + (self.input[self.position] - b'0');
                    self.position += 1;
                    count += 1;
                }
                _ => break,
            }
        }

        Ok(value)
    }

    fn read_hex_string(&mut self) -> ParseResult<Option<Token>> {
        self.position += 1; // Skip opening '<'
        let mut result = Vec::new();
        let mut nibble = None;

        while self.position < self.input.len() {
            let ch = self.input[self.position];

            match ch {
                b'>' => {
                    self.position += 1;
                    // Handle odd number of hex digits
                    if let Some(n) = nibble {
                        result.push(n << 4);
                    }
                    return Ok(Some(Token::HexString(result)));
                }
                b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f' => {
                    let digit = if ch <= b'9' {
                        ch - b'0'
                    } else if ch <= b'F' {
                        ch - b'A' + 10
                    } else {
                        ch - b'a' + 10
                    };

                    if let Some(n) = nibble {
                        result.push((n << 4) | digit);
                        nibble = None;
                    } else {
                        nibble = Some(digit);
                    }
                    self.position += 1;
                }
                b' ' | b'\t' | b'\r' | b'\n' | b'\x0C' => {
                    // Skip whitespace in hex strings
                    self.position += 1;
                }
                _ => {
                    return Err(ParseError::SyntaxError {
                        position: self.position,
                        message: format!("Invalid character in hex string: {:?}", ch as char),
                    });
                }
            }
        }

        Err(ParseError::SyntaxError {
            position: self.position,
            message: "Unterminated hex string".to_string(),
        })
    }

    fn read_name(&mut self) -> ParseResult<Option<Token>> {
        self.position += 1; // Skip '/'
        let start = self.position;

        while self.position < self.input.len() {
            let ch = self.input[self.position];
            match ch {
                b' ' | b'\t' | b'\r' | b'\n' | b'\x0C' | b'(' | b')' | b'<' | b'>' | b'['
                | b']' | b'{' | b'}' | b'/' | b'%' => break,
                b'#' => {
                    // Handle hex escape in name
                    self.position += 1;
                    if self.position + 1 < self.input.len() {
                        self.position += 2;
                    }
                }
                _ => self.position += 1,
            }
        }

        let name_bytes = &self.input[start..self.position];
        let name = self.decode_name(name_bytes)?;
        Ok(Some(Token::Name(name)))
    }

    fn decode_name(&self, bytes: &[u8]) -> ParseResult<String> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] == b'#' && i + 2 < bytes.len() {
                // Hex escape
                let hex_str = std::str::from_utf8(&bytes[i + 1..i + 3]).map_err(|_| {
                    ParseError::SyntaxError {
                        position: self.position,
                        message: "Invalid hex escape in name".to_string(),
                    }
                })?;
                let value =
                    u8::from_str_radix(hex_str, 16).map_err(|_| ParseError::SyntaxError {
                        position: self.position,
                        message: "Invalid hex escape in name".to_string(),
                    })?;
                result.push(value);
                i += 3;
            } else {
                result.push(bytes[i]);
                i += 1;
            }
        }

        String::from_utf8(result).map_err(|_| ParseError::SyntaxError {
            position: self.position,
            message: "Invalid UTF-8 in name".to_string(),
        })
    }

    fn read_operator(&mut self) -> ParseResult<Option<Token>> {
        let start = self.position;

        while self.position < self.input.len() {
            let ch = self.input[self.position];
            match ch {
                b' ' | b'\t' | b'\r' | b'\n' | b'\x0C' | b'(' | b')' | b'<' | b'>' | b'['
                | b']' | b'{' | b'}' | b'/' | b'%' => break,
                _ => self.position += 1,
            }
        }

        let op_bytes = &self.input[start..self.position];
        let op = std::str::from_utf8(op_bytes).map_err(|_| ParseError::SyntaxError {
            position: start,
            message: "Invalid operator".to_string(),
        })?;

        Ok(Some(Token::Operator(op.to_string())))
    }
}

/// Content stream parser
pub struct ContentParser {
    tokens: Vec<Token>,
    position: usize,
}

impl ContentParser {
    /// Create a new content parser
    pub fn new(_content: &[u8]) -> Self {
        Self {
            tokens: Vec::new(),
            position: 0,
        }
    }

    /// Parse a content stream into a vector of operators (static method for tests)
    pub fn parse(content: &[u8]) -> ParseResult<Vec<ContentOperation>> {
        Self::parse_content(content)
    }

    /// Parse a content stream into a vector of operators
    pub fn parse_content(content: &[u8]) -> ParseResult<Vec<ContentOperation>> {
        let mut tokenizer = ContentTokenizer::new(content);
        let mut tokens = Vec::new();

        // Tokenize the entire stream
        while let Some(token) = tokenizer.next_token()? {
            tokens.push(token);
        }

        let mut parser = Self {
            tokens,
            position: 0,
        };

        parser.parse_operators()
    }

    fn parse_operators(&mut self) -> ParseResult<Vec<ContentOperation>> {
        let mut operators = Vec::new();
        let mut operand_stack: Vec<Token> = Vec::new();

        while self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;

            match &token {
                Token::Operator(op) => {
                    let operator = self.parse_operator(op, &mut operand_stack)?;
                    operators.push(operator);
                }
                _ => {
                    // Not an operator, push to operand stack
                    operand_stack.push(token);
                }
            }
        }

        Ok(operators)
    }

    fn parse_operator(
        &mut self,
        op: &str,
        operands: &mut Vec<Token>,
    ) -> ParseResult<ContentOperation> {
        let operator = match op {
            // Text object operators
            "BT" => ContentOperation::BeginText,
            "ET" => ContentOperation::EndText,

            // Text state operators
            "Tc" => {
                let spacing = self.pop_number(operands)?;
                ContentOperation::SetCharSpacing(spacing)
            }
            "Tw" => {
                let spacing = self.pop_number(operands)?;
                ContentOperation::SetWordSpacing(spacing)
            }
            "Tz" => {
                let scale = self.pop_number(operands)?;
                ContentOperation::SetHorizontalScaling(scale)
            }
            "TL" => {
                let leading = self.pop_number(operands)?;
                ContentOperation::SetLeading(leading)
            }
            "Tf" => {
                let size = self.pop_number(operands)?;
                let font = self.pop_name(operands)?;
                ContentOperation::SetFont(font, size)
            }
            "Tr" => {
                let mode = self.pop_integer(operands)?;
                ContentOperation::SetTextRenderMode(mode)
            }
            "Ts" => {
                let rise = self.pop_number(operands)?;
                ContentOperation::SetTextRise(rise)
            }

            // Text positioning operators
            "Td" => {
                let ty = self.pop_number(operands)?;
                let tx = self.pop_number(operands)?;
                ContentOperation::MoveText(tx, ty)
            }
            "TD" => {
                let ty = self.pop_number(operands)?;
                let tx = self.pop_number(operands)?;
                ContentOperation::MoveTextSetLeading(tx, ty)
            }
            "Tm" => {
                let f = self.pop_number(operands)?;
                let e = self.pop_number(operands)?;
                let d = self.pop_number(operands)?;
                let c = self.pop_number(operands)?;
                let b = self.pop_number(operands)?;
                let a = self.pop_number(operands)?;
                ContentOperation::SetTextMatrix(a, b, c, d, e, f)
            }
            "T*" => ContentOperation::NextLine,

            // Text showing operators
            "Tj" => {
                let text = self.pop_string(operands)?;
                ContentOperation::ShowText(text)
            }
            "TJ" => {
                let array = self.pop_array(operands)?;
                let elements = self.parse_text_array(array)?;
                ContentOperation::ShowTextArray(elements)
            }
            "'" => {
                let text = self.pop_string(operands)?;
                ContentOperation::NextLineShowText(text)
            }
            "\"" => {
                let text = self.pop_string(operands)?;
                let aw = self.pop_number(operands)?;
                let ac = self.pop_number(operands)?;
                ContentOperation::SetSpacingNextLineShowText(ac, aw, text)
            }

            // Graphics state operators
            "q" => ContentOperation::SaveGraphicsState,
            "Q" => ContentOperation::RestoreGraphicsState,
            "cm" => {
                let f = self.pop_number(operands)?;
                let e = self.pop_number(operands)?;
                let d = self.pop_number(operands)?;
                let c = self.pop_number(operands)?;
                let b = self.pop_number(operands)?;
                let a = self.pop_number(operands)?;
                ContentOperation::SetTransformMatrix(a, b, c, d, e, f)
            }
            "w" => {
                let width = self.pop_number(operands)?;
                ContentOperation::SetLineWidth(width)
            }
            "J" => {
                let cap = self.pop_integer(operands)?;
                ContentOperation::SetLineCap(cap)
            }
            "j" => {
                let join = self.pop_integer(operands)?;
                ContentOperation::SetLineJoin(join)
            }
            "M" => {
                let limit = self.pop_number(operands)?;
                ContentOperation::SetMiterLimit(limit)
            }
            "d" => {
                let phase = self.pop_number(operands)?;
                let array = self.pop_array(operands)?;
                let pattern = self.parse_dash_array(array)?;
                ContentOperation::SetDashPattern(pattern, phase)
            }
            "ri" => {
                let intent = self.pop_name(operands)?;
                ContentOperation::SetIntent(intent)
            }
            "i" => {
                let flatness = self.pop_number(operands)?;
                ContentOperation::SetFlatness(flatness)
            }
            "gs" => {
                let name = self.pop_name(operands)?;
                ContentOperation::SetGraphicsStateParams(name)
            }

            // Path construction operators
            "m" => {
                let y = self.pop_number(operands)?;
                let x = self.pop_number(operands)?;
                ContentOperation::MoveTo(x, y)
            }
            "l" => {
                let y = self.pop_number(operands)?;
                let x = self.pop_number(operands)?;
                ContentOperation::LineTo(x, y)
            }
            "c" => {
                let y3 = self.pop_number(operands)?;
                let x3 = self.pop_number(operands)?;
                let y2 = self.pop_number(operands)?;
                let x2 = self.pop_number(operands)?;
                let y1 = self.pop_number(operands)?;
                let x1 = self.pop_number(operands)?;
                ContentOperation::CurveTo(x1, y1, x2, y2, x3, y3)
            }
            "v" => {
                let y3 = self.pop_number(operands)?;
                let x3 = self.pop_number(operands)?;
                let y2 = self.pop_number(operands)?;
                let x2 = self.pop_number(operands)?;
                ContentOperation::CurveToV(x2, y2, x3, y3)
            }
            "y" => {
                let y3 = self.pop_number(operands)?;
                let x3 = self.pop_number(operands)?;
                let y1 = self.pop_number(operands)?;
                let x1 = self.pop_number(operands)?;
                ContentOperation::CurveToY(x1, y1, x3, y3)
            }
            "h" => ContentOperation::ClosePath,
            "re" => {
                let height = self.pop_number(operands)?;
                let width = self.pop_number(operands)?;
                let y = self.pop_number(operands)?;
                let x = self.pop_number(operands)?;
                ContentOperation::Rectangle(x, y, width, height)
            }

            // Path painting operators
            "S" => ContentOperation::Stroke,
            "s" => ContentOperation::CloseStroke,
            "f" | "F" => ContentOperation::Fill,
            "f*" => ContentOperation::FillEvenOdd,
            "B" => ContentOperation::FillStroke,
            "B*" => ContentOperation::FillStrokeEvenOdd,
            "b" => ContentOperation::CloseFillStroke,
            "b*" => ContentOperation::CloseFillStrokeEvenOdd,
            "n" => ContentOperation::EndPath,

            // Clipping path operators
            "W" => ContentOperation::Clip,
            "W*" => ContentOperation::ClipEvenOdd,

            // Color operators
            "CS" => {
                let name = self.pop_name(operands)?;
                ContentOperation::SetStrokingColorSpace(name)
            }
            "cs" => {
                let name = self.pop_name(operands)?;
                ContentOperation::SetNonStrokingColorSpace(name)
            }
            "SC" | "SCN" => {
                let components = self.pop_color_components(operands)?;
                ContentOperation::SetStrokingColor(components)
            }
            "sc" | "scn" => {
                let components = self.pop_color_components(operands)?;
                ContentOperation::SetNonStrokingColor(components)
            }
            "G" => {
                let gray = self.pop_number(operands)?;
                ContentOperation::SetStrokingGray(gray)
            }
            "g" => {
                let gray = self.pop_number(operands)?;
                ContentOperation::SetNonStrokingGray(gray)
            }
            "RG" => {
                let b = self.pop_number(operands)?;
                let g = self.pop_number(operands)?;
                let r = self.pop_number(operands)?;
                ContentOperation::SetStrokingRGB(r, g, b)
            }
            "rg" => {
                let b = self.pop_number(operands)?;
                let g = self.pop_number(operands)?;
                let r = self.pop_number(operands)?;
                ContentOperation::SetNonStrokingRGB(r, g, b)
            }
            "K" => {
                let k = self.pop_number(operands)?;
                let y = self.pop_number(operands)?;
                let m = self.pop_number(operands)?;
                let c = self.pop_number(operands)?;
                ContentOperation::SetStrokingCMYK(c, m, y, k)
            }
            "k" => {
                let k = self.pop_number(operands)?;
                let y = self.pop_number(operands)?;
                let m = self.pop_number(operands)?;
                let c = self.pop_number(operands)?;
                ContentOperation::SetNonStrokingCMYK(c, m, y, k)
            }

            // Shading operators
            "sh" => {
                let name = self.pop_name(operands)?;
                ContentOperation::ShadingFill(name)
            }

            // XObject operators
            "Do" => {
                let name = self.pop_name(operands)?;
                ContentOperation::PaintXObject(name)
            }

            // Marked content operators
            "BMC" => {
                let tag = self.pop_name(operands)?;
                ContentOperation::BeginMarkedContent(tag)
            }
            "BDC" => {
                let props = self.pop_dict_or_name(operands)?;
                let tag = self.pop_name(operands)?;
                ContentOperation::BeginMarkedContentWithProps(tag, props)
            }
            "EMC" => ContentOperation::EndMarkedContent,
            "MP" => {
                let tag = self.pop_name(operands)?;
                ContentOperation::DefineMarkedContentPoint(tag)
            }
            "DP" => {
                let props = self.pop_dict_or_name(operands)?;
                let tag = self.pop_name(operands)?;
                ContentOperation::DefineMarkedContentPointWithProps(tag, props)
            }

            // Compatibility operators
            "BX" => ContentOperation::BeginCompatibility,
            "EX" => ContentOperation::EndCompatibility,

            // Inline images are handled specially
            "BI" => {
                operands.clear(); // Clear any remaining operands
                self.parse_inline_image()?
            }

            _ => {
                return Err(ParseError::SyntaxError {
                    position: self.position,
                    message: format!("Unknown operator: {op}"),
                });
            }
        };

        operands.clear(); // Clear operands after processing
        Ok(operator)
    }

    // Helper methods for popping operands
    fn pop_number(&self, operands: &mut Vec<Token>) -> ParseResult<f32> {
        match operands.pop() {
            Some(Token::Number(n)) => Ok(n),
            Some(Token::Integer(i)) => Ok(i as f32),
            _ => Err(ParseError::SyntaxError {
                position: self.position,
                message: "Expected number operand".to_string(),
            }),
        }
    }

    fn pop_integer(&self, operands: &mut Vec<Token>) -> ParseResult<i32> {
        match operands.pop() {
            Some(Token::Integer(i)) => Ok(i),
            _ => Err(ParseError::SyntaxError {
                position: self.position,
                message: "Expected integer operand".to_string(),
            }),
        }
    }

    fn pop_name(&self, operands: &mut Vec<Token>) -> ParseResult<String> {
        match operands.pop() {
            Some(Token::Name(n)) => Ok(n),
            _ => Err(ParseError::SyntaxError {
                position: self.position,
                message: "Expected name operand".to_string(),
            }),
        }
    }

    fn pop_string(&self, operands: &mut Vec<Token>) -> ParseResult<Vec<u8>> {
        match operands.pop() {
            Some(Token::String(s)) => Ok(s),
            Some(Token::HexString(s)) => Ok(s),
            _ => Err(ParseError::SyntaxError {
                position: self.position,
                message: "Expected string operand".to_string(),
            }),
        }
    }

    fn pop_array(&self, operands: &mut Vec<Token>) -> ParseResult<Vec<Token>> {
        let mut array = Vec::new();
        let mut found_start = false;

        // Pop tokens until we find ArrayStart
        while let Some(token) = operands.pop() {
            match token {
                Token::ArrayStart => {
                    found_start = true;
                    break;
                }
                _ => array.push(token),
            }
        }

        if !found_start {
            return Err(ParseError::SyntaxError {
                position: self.position,
                message: "Expected array".to_string(),
            });
        }

        array.reverse(); // We collected in reverse order
        Ok(array)
    }

    fn pop_dict_or_name(&self, operands: &mut Vec<Token>) -> ParseResult<HashMap<String, String>> {
        // For now, we'll just return an empty map
        // Full dictionary parsing would be more complex
        operands.pop();
        Ok(HashMap::new())
    }

    fn pop_color_components(&self, operands: &mut Vec<Token>) -> ParseResult<Vec<f32>> {
        let mut components = Vec::new();

        // Pop all numeric values from the stack
        while let Some(token) = operands.last() {
            match token {
                Token::Number(n) => {
                    components.push(*n);
                    operands.pop();
                }
                Token::Integer(i) => {
                    components.push(*i as f32);
                    operands.pop();
                }
                _ => break,
            }
        }

        components.reverse();
        Ok(components)
    }

    fn parse_text_array(&self, tokens: Vec<Token>) -> ParseResult<Vec<TextElement>> {
        let mut elements = Vec::new();

        for token in tokens {
            match token {
                Token::String(s) | Token::HexString(s) => {
                    elements.push(TextElement::Text(s));
                }
                Token::Number(n) => {
                    elements.push(TextElement::Spacing(n));
                }
                Token::Integer(i) => {
                    elements.push(TextElement::Spacing(i as f32));
                }
                _ => {
                    return Err(ParseError::SyntaxError {
                        position: self.position,
                        message: "Invalid element in text array".to_string(),
                    });
                }
            }
        }

        Ok(elements)
    }

    fn parse_dash_array(&self, tokens: Vec<Token>) -> ParseResult<Vec<f32>> {
        let mut pattern = Vec::new();

        for token in tokens {
            match token {
                Token::Number(n) => pattern.push(n),
                Token::Integer(i) => pattern.push(i as f32),
                _ => {
                    return Err(ParseError::SyntaxError {
                        position: self.position,
                        message: "Invalid element in dash array".to_string(),
                    });
                }
            }
        }

        Ok(pattern)
    }

    fn parse_inline_image(&mut self) -> ParseResult<ContentOperation> {
        // For now, we'll skip inline images
        // This would require parsing the image dictionary and data
        // Skip tokens until we find EI
        while self.position < self.tokens.len() {
            if let Token::Operator(op) = &self.tokens[self.position] {
                if op == "EI" {
                    self.position += 1;
                    break;
                }
            }
            self.position += 1;
        }

        Ok(ContentOperation::BeginInlineImage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_numbers() {
        let input = b"123 -45 3.14 -0.5 .5";
        let mut tokenizer = ContentTokenizer::new(input);

        assert_eq!(tokenizer.next_token().unwrap(), Some(Token::Integer(123)));
        assert_eq!(tokenizer.next_token().unwrap(), Some(Token::Integer(-45)));
        assert_eq!(tokenizer.next_token().unwrap(), Some(Token::Number(3.14)));
        assert_eq!(tokenizer.next_token().unwrap(), Some(Token::Number(-0.5)));
        assert_eq!(tokenizer.next_token().unwrap(), Some(Token::Number(0.5)));
        assert_eq!(tokenizer.next_token().unwrap(), None);
    }

    #[test]
    fn test_tokenize_strings() {
        let input = b"(Hello World) (Hello\\nWorld) (Nested (paren))";
        let mut tokenizer = ContentTokenizer::new(input);

        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::String(b"Hello World".to_vec()))
        );
        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::String(b"Hello\nWorld".to_vec()))
        );
        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::String(b"Nested (paren)".to_vec()))
        );
    }

    #[test]
    fn test_tokenize_hex_strings() {
        let input = b"<48656C6C6F> <48 65 6C 6C 6F>";
        let mut tokenizer = ContentTokenizer::new(input);

        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::HexString(b"Hello".to_vec()))
        );
        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::HexString(b"Hello".to_vec()))
        );
    }

    #[test]
    fn test_tokenize_names() {
        let input = b"/Name /Name#20with#20spaces /A#42C";
        let mut tokenizer = ContentTokenizer::new(input);

        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::Name("Name".to_string()))
        );
        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::Name("Name with spaces".to_string()))
        );
        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::Name("ABC".to_string()))
        );
    }

    #[test]
    fn test_tokenize_operators() {
        let input = b"BT Tj ET q Q";
        let mut tokenizer = ContentTokenizer::new(input);

        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::Operator("BT".to_string()))
        );
        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::Operator("Tj".to_string()))
        );
        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::Operator("ET".to_string()))
        );
        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::Operator("q".to_string()))
        );
        assert_eq!(
            tokenizer.next_token().unwrap(),
            Some(Token::Operator("Q".to_string()))
        );
    }

    #[test]
    fn test_parse_text_operators() {
        let content = b"BT /F1 12 Tf 100 200 Td (Hello World) Tj ET";
        let operators = ContentParser::parse(content).unwrap();

        assert_eq!(operators.len(), 5);
        assert_eq!(operators[0], ContentOperation::BeginText);
        assert_eq!(
            operators[1],
            ContentOperation::SetFont("F1".to_string(), 12.0)
        );
        assert_eq!(operators[2], ContentOperation::MoveText(100.0, 200.0));
        assert_eq!(
            operators[3],
            ContentOperation::ShowText(b"Hello World".to_vec())
        );
        assert_eq!(operators[4], ContentOperation::EndText);
    }

    #[test]
    fn test_parse_graphics_operators() {
        let content = b"q 1 0 0 1 50 50 cm 2 w 0 0 100 100 re S Q";
        let operators = ContentParser::parse(content).unwrap();

        assert_eq!(operators.len(), 6);
        assert_eq!(operators[0], ContentOperation::SaveGraphicsState);
        assert_eq!(
            operators[1],
            ContentOperation::SetTransformMatrix(1.0, 0.0, 0.0, 1.0, 50.0, 50.0)
        );
        assert_eq!(operators[2], ContentOperation::SetLineWidth(2.0));
        assert_eq!(
            operators[3],
            ContentOperation::Rectangle(0.0, 0.0, 100.0, 100.0)
        );
        assert_eq!(operators[4], ContentOperation::Stroke);
        assert_eq!(operators[5], ContentOperation::RestoreGraphicsState);
    }

    #[test]
    fn test_parse_color_operators() {
        let content = b"0.5 g 1 0 0 rg 0 0 0 1 k";
        let operators = ContentParser::parse(content).unwrap();

        assert_eq!(operators.len(), 3);
        assert_eq!(operators[0], ContentOperation::SetNonStrokingGray(0.5));
        assert_eq!(
            operators[1],
            ContentOperation::SetNonStrokingRGB(1.0, 0.0, 0.0)
        );
        assert_eq!(
            operators[2],
            ContentOperation::SetNonStrokingCMYK(0.0, 0.0, 0.0, 1.0)
        );
    }
}
