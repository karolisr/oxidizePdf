//! Text extraction from PDF content streams
//!
//! This module provides functionality to extract text from PDF pages,
//! handling text positioning, transformations, and basic encodings.

use crate::parser::content::{ContentOperation, ContentParser, TextElement};
use crate::parser::document::PdfDocument;
use crate::parser::ParseResult;
use std::io::{Read, Seek};

/// Text extraction options
#[derive(Debug, Clone)]
pub struct ExtractionOptions {
    /// Preserve the original layout (spacing and positioning)
    pub preserve_layout: bool,
    /// Minimum space width to insert space character (in text space units)
    pub space_threshold: f64,
    /// Minimum vertical distance to insert newline (in text space units)
    pub newline_threshold: f64,
}

impl Default for ExtractionOptions {
    fn default() -> Self {
        Self {
            preserve_layout: false,
            space_threshold: 0.2,
            newline_threshold: 10.0,
        }
    }
}

/// Extracted text with position information
#[derive(Debug, Clone)]
pub struct ExtractedText {
    /// The extracted text content
    pub text: String,
    /// Text fragments with position information (if preserve_layout is true)
    pub fragments: Vec<TextFragment>,
}

/// A fragment of text with position information
#[derive(Debug, Clone)]
pub struct TextFragment {
    /// Text content
    pub text: String,
    /// X position in page coordinates
    pub x: f64,
    /// Y position in page coordinates
    pub y: f64,
    /// Width of the text
    pub width: f64,
    /// Height of the text
    pub height: f64,
    /// Font size
    pub font_size: f64,
}

/// Text extraction state
struct TextState {
    /// Current text matrix
    text_matrix: [f64; 6],
    /// Current text line matrix
    text_line_matrix: [f64; 6],
    /// Current transformation matrix (CTM)
    #[allow(dead_code)]
    ctm: [f64; 6],
    /// Text leading (line spacing)
    leading: f64,
    /// Character spacing
    char_space: f64,
    /// Word spacing
    word_space: f64,
    /// Horizontal scaling
    horizontal_scale: f64,
    /// Text rise
    text_rise: f64,
    /// Current font size
    font_size: f64,
    /// Current font name
    font_name: Option<String>,
    /// Render mode (0 = fill, 1 = stroke, etc.)
    render_mode: u8,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            text_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            text_line_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            leading: 0.0,
            char_space: 0.0,
            word_space: 0.0,
            horizontal_scale: 100.0,
            text_rise: 0.0,
            font_size: 0.0,
            font_name: None,
            render_mode: 0,
        }
    }
}

/// Text extractor for PDF pages
pub struct TextExtractor {
    options: ExtractionOptions,
}

impl TextExtractor {
    /// Create a new text extractor with default options
    pub fn new() -> Self {
        Self {
            options: ExtractionOptions::default(),
        }
    }

    /// Create a text extractor with custom options
    pub fn with_options(options: ExtractionOptions) -> Self {
        Self { options }
    }

    /// Extract text from a PDF document
    pub fn extract_from_document<R: Read + Seek>(
        &self,
        document: &PdfDocument<R>,
    ) -> ParseResult<Vec<ExtractedText>> {
        let page_count = document.page_count()?;
        let mut results = Vec::new();

        for i in 0..page_count {
            let text = self.extract_from_page(document, i)?;
            results.push(text);
        }

        Ok(results)
    }

    /// Extract text from a specific page
    pub fn extract_from_page<R: Read + Seek>(
        &self,
        document: &PdfDocument<R>,
        page_index: u32,
    ) -> ParseResult<ExtractedText> {
        // Get the page
        let page = document.get_page(page_index)?;

        // Get content streams
        let streams = page.content_streams_with_document(document)?;

        let mut extracted_text = String::new();
        let mut fragments = Vec::new();
        let mut state = TextState::default();
        let mut in_text_object = false;
        let mut last_x = 0.0;
        let mut last_y = 0.0;

        // Process each content stream
        for stream_data in streams {
            let operations = ContentParser::parse_content(&stream_data)?;

            for op in operations {
                match op {
                    ContentOperation::BeginText => {
                        in_text_object = true;
                        // Reset text matrix to identity
                        state.text_matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
                        state.text_line_matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
                    }

                    ContentOperation::EndText => {
                        in_text_object = false;
                    }

                    ContentOperation::SetTextMatrix(a, b, c, d, e, f) => {
                        state.text_matrix =
                            [a as f64, b as f64, c as f64, d as f64, e as f64, f as f64];
                        state.text_line_matrix =
                            [a as f64, b as f64, c as f64, d as f64, e as f64, f as f64];
                    }

                    ContentOperation::MoveText(tx, ty) => {
                        // Update text matrix by translation
                        let new_matrix = multiply_matrix(
                            &[1.0, 0.0, 0.0, 1.0, tx as f64, ty as f64],
                            &state.text_line_matrix,
                        );
                        state.text_matrix = new_matrix;
                        state.text_line_matrix = new_matrix;
                    }

                    ContentOperation::NextLine => {
                        // Move to next line using current leading
                        let new_matrix = multiply_matrix(
                            &[1.0, 0.0, 0.0, 1.0, 0.0, -state.leading],
                            &state.text_line_matrix,
                        );
                        state.text_matrix = new_matrix;
                        state.text_line_matrix = new_matrix;
                    }

                    ContentOperation::ShowText(text) => {
                        if in_text_object {
                            let text_bytes = &text;
                            let decoded = self.decode_text(text_bytes, &state)?;

                            // Calculate position
                            let (x, y) = transform_point(0.0, 0.0, &state.text_matrix);

                            // Add spacing based on position change
                            if !extracted_text.is_empty() {
                                let dx = x - last_x;
                                let dy = (y - last_y).abs();

                                if dy > self.options.newline_threshold {
                                    extracted_text.push('\n');
                                } else if dx > self.options.space_threshold * state.font_size {
                                    extracted_text.push(' ');
                                }
                            }

                            extracted_text.push_str(&decoded);

                            if self.options.preserve_layout {
                                fragments.push(TextFragment {
                                    text: decoded.clone(),
                                    x,
                                    y,
                                    width: calculate_text_width(&decoded, state.font_size),
                                    height: state.font_size,
                                    font_size: state.font_size,
                                });
                            }

                            // Update position for next text
                            last_x = x + calculate_text_width(&decoded, state.font_size);
                            last_y = y;

                            // Update text matrix for next show operation
                            let text_width = calculate_text_width(&decoded, state.font_size);
                            let tx = text_width * state.horizontal_scale / 100.0;
                            state.text_matrix =
                                multiply_matrix(&[1.0, 0.0, 0.0, 1.0, tx, 0.0], &state.text_matrix);
                        }
                    }

                    ContentOperation::ShowTextArray(array) => {
                        if in_text_object {
                            for item in array {
                                match item {
                                    TextElement::Text(text_bytes) => {
                                        let decoded = self.decode_text(&text_bytes, &state)?;
                                        extracted_text.push_str(&decoded);

                                        // Update text matrix
                                        let text_width =
                                            calculate_text_width(&decoded, state.font_size);
                                        let tx = text_width * state.horizontal_scale / 100.0;
                                        state.text_matrix = multiply_matrix(
                                            &[1.0, 0.0, 0.0, 1.0, tx, 0.0],
                                            &state.text_matrix,
                                        );
                                    }
                                    TextElement::Spacing(adjustment) => {
                                        // Text position adjustment (negative = move left)
                                        let tx = -(adjustment as f64) / 1000.0 * state.font_size;
                                        state.text_matrix = multiply_matrix(
                                            &[1.0, 0.0, 0.0, 1.0, tx, 0.0],
                                            &state.text_matrix,
                                        );
                                    }
                                }
                            }
                        }
                    }

                    ContentOperation::SetFont(name, size) => {
                        state.font_name = Some(name);
                        state.font_size = size as f64;
                    }

                    ContentOperation::SetLeading(leading) => {
                        state.leading = leading as f64;
                    }

                    ContentOperation::SetCharSpacing(spacing) => {
                        state.char_space = spacing as f64;
                    }

                    ContentOperation::SetWordSpacing(spacing) => {
                        state.word_space = spacing as f64;
                    }

                    ContentOperation::SetHorizontalScaling(scale) => {
                        state.horizontal_scale = scale as f64;
                    }

                    ContentOperation::SetTextRise(rise) => {
                        state.text_rise = rise as f64;
                    }

                    ContentOperation::SetTextRenderMode(mode) => {
                        state.render_mode = mode as u8;
                    }

                    _ => {
                        // Other operations don't affect text extraction
                    }
                }
            }
        }

        Ok(ExtractedText {
            text: extracted_text,
            fragments,
        })
    }

    /// Decode text using the current font encoding
    fn decode_text(&self, text: &[u8], _state: &TextState) -> ParseResult<String> {
        // TODO: Get actual font encoding from state
        // For now, assume WinAnsiEncoding which is the default for most PDFs
        use crate::text::encoding::TextEncoding;

        let encoding = TextEncoding::WinAnsiEncoding;
        Ok(encoding.decode(text))
    }
}

impl Default for TextExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Multiply two transformation matrices
fn multiply_matrix(a: &[f64; 6], b: &[f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

/// Transform a point using a transformation matrix
fn transform_point(x: f64, y: f64, matrix: &[f64; 6]) -> (f64, f64) {
    let tx = matrix[0] * x + matrix[2] * y + matrix[4];
    let ty = matrix[1] * x + matrix[3] * y + matrix[5];
    (tx, ty)
}

/// Calculate approximate text width (simplified)
fn calculate_text_width(text: &str, font_size: f64) -> f64 {
    // Approximate: assume average character width is 0.5 * font_size
    text.len() as f64 * font_size * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiplication() {
        let identity = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let translation = [1.0, 0.0, 0.0, 1.0, 10.0, 20.0];

        let result = multiply_matrix(&identity, &translation);
        assert_eq!(result, translation);

        let result2 = multiply_matrix(&translation, &identity);
        assert_eq!(result2, translation);
    }

    #[test]
    fn test_transform_point() {
        let translation = [1.0, 0.0, 0.0, 1.0, 10.0, 20.0];
        let (x, y) = transform_point(5.0, 5.0, &translation);
        assert_eq!(x, 15.0);
        assert_eq!(y, 25.0);
    }
}
