//! Test PDF Builder
//!
//! A builder for creating test PDFs with specific characteristics.

use std::collections::HashMap;

/// PDF version to generate
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum PdfVersion {
    V1_0,
    V1_1,
    V1_2,
    V1_3,
    V1_4,
    V1_5,
    V1_6,
    V1_7,
    V2_0,
}

impl std::fmt::Display for PdfVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let version = match self {
            PdfVersion::V1_0 => "1.0",
            PdfVersion::V1_1 => "1.1",
            PdfVersion::V1_2 => "1.2",
            PdfVersion::V1_3 => "1.3",
            PdfVersion::V1_4 => "1.4",
            PdfVersion::V1_5 => "1.5",
            PdfVersion::V1_6 => "1.6",
            PdfVersion::V1_7 => "1.7",
            PdfVersion::V2_0 => "2.0",
        };
        write!(f, "{version}")
    }
}

/// Builder for creating test PDFs
pub struct TestPdfBuilder {
    version: PdfVersion,
    objects: Vec<PdfObject>,
    pages: Vec<PageContent>,
    info: HashMap<String, String>,
    include_binary_marker: bool,
    compress_streams: bool,
    use_xref_stream: bool,
    #[allow(dead_code)]
    linearized: bool,
}

#[derive(Clone)]
struct PdfObject {
    number: u32,
    generation: u16,
    content: String,
}

#[derive(Clone)]
struct PageContent {
    width: f32,
    height: f32,
    content_stream: String,
    resources: HashMap<String, String>,
}

impl TestPdfBuilder {
    /// Create a new PDF builder with default settings
    pub fn new() -> Self {
        Self {
            version: PdfVersion::V1_4,
            objects: Vec::new(),
            pages: Vec::new(),
            info: HashMap::new(),
            include_binary_marker: true,
            compress_streams: false,
            use_xref_stream: false,
            linearized: false,
        }
    }

    /// Create a minimal valid PDF
    pub fn minimal() -> Self {
        let mut builder = Self::new();
        builder.add_empty_page(612.0, 792.0);
        builder
    }

    /// Set PDF version
    pub fn with_version(mut self, version: PdfVersion) -> Self {
        self.version = version;
        self
    }

    /// Add document info
    pub fn with_info(mut self, key: &str, value: &str) -> Self {
        self.info.insert(key.to_string(), value.to_string());
        self
    }

    /// Add title
    pub fn with_title(self, title: &str) -> Self {
        self.with_info("Title", title)
    }

    /// Add author
    pub fn with_author(self, author: &str) -> Self {
        self.with_info("Author", author)
    }

    /// Add creator
    pub fn with_creator(self, creator: &str) -> Self {
        self.with_info("Creator", creator)
    }

    /// Add producer
    pub fn with_producer(self, producer: &str) -> Self {
        self.with_info("Producer", producer)
    }

    /// Add an empty page
    pub fn add_empty_page(&mut self, width: f32, height: f32) -> &mut Self {
        self.pages.push(PageContent {
            width,
            height,
            content_stream: String::new(),
            resources: HashMap::new(),
        });
        self
    }

    /// Add a page with text
    pub fn add_text_page(&mut self, text: &str, font_size: f32) -> &mut Self {
        let content = format!(
            "BT\n/F1 {} Tf\n100 700 Td\n({}) Tj\nET",
            font_size,
            escape_pdf_string(text)
        );

        let mut resources = HashMap::new();
        resources.insert(
            "Font".to_string(),
            "<< /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >>".to_string(),
        );

        self.pages.push(PageContent {
            width: 612.0,
            height: 792.0,
            content_stream: content,
            resources,
        });
        self
    }

    /// Add a page with graphics
    pub fn add_graphics_page(&mut self) -> &mut Self {
        let content = "q\n\
                      1 0 0 RG\n\
                      2 w\n\
                      100 100 400 600 re\n\
                      S\n\
                      0 0 1 RG\n\
                      200 200 200 200 re\n\
                      f\n\
                      Q"
        .to_string();

        self.pages.push(PageContent {
            width: 612.0,
            height: 792.0,
            content_stream: content,
            resources: HashMap::new(),
        });
        self
    }

    /// Enable stream compression
    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress_streams = compress;
        self
    }

    /// Use cross-reference streams (PDF 1.5+)
    pub fn with_xref_stream(mut self, use_xref: bool) -> Self {
        self.use_xref_stream = use_xref;
        if use_xref && self.version < PdfVersion::V1_5 {
            self.version = PdfVersion::V1_5;
        }
        self
    }

    /// Create invalid xref table
    pub fn with_invalid_xref(self) -> Self {
        // This will be handled in the build phase
        self
    }

    /// Create circular reference
    pub fn with_circular_reference(mut self) -> Self {
        // Add objects that reference each other
        self.objects.push(PdfObject {
            number: 100,
            generation: 0,
            content: "<< /Type /Test /Next 101 0 R >>".to_string(),
        });
        self.objects.push(PdfObject {
            number: 101,
            generation: 0,
            content: "<< /Type /Test /Next 100 0 R >>".to_string(),
        });
        self
    }

    /// Build the PDF
    pub fn build(&self) -> Vec<u8> {
        let mut pdf = Vec::new();
        let mut xref_positions = Vec::new();

        // Header
        pdf.extend_from_slice(format!("%PDF-{}\n", self.version).as_bytes());

        // Binary marker
        if self.include_binary_marker {
            pdf.extend_from_slice(b"%\xE2\xE3\xCF\xD3\n");
        }

        // Build objects
        let mut object_num = 1;
        let catalog_obj = object_num;
        let pages_obj = object_num + 1;
        let mut info_obj = 0;

        // Catalog object
        xref_positions.push(pdf.len());
        pdf.extend_from_slice(
            format!("{catalog_obj} 0 obj\n<< /Type /Catalog /Pages {pages_obj} 0 R >>\nendobj\n")
                .as_bytes(),
        );
        object_num += 1;

        // Pages object
        let page_refs: Vec<String> = if !self.pages.is_empty() {
            (0..self.pages.len())
                .map(|i| format!("{} 0 R", pages_obj + 1 + i as u32))
                .collect()
        } else {
            vec![]
        };

        xref_positions.push(pdf.len());
        pdf.extend_from_slice(
            format!(
                "{} 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
                pages_obj,
                page_refs.join(" "),
                self.pages.len()
            )
            .as_bytes(),
        );
        object_num += 1;

        // Individual pages
        for (i, page) in self.pages.iter().enumerate() {
            xref_positions.push(pdf.len());

            let mut page_dict = format!(
                "<< /Type /Page /Parent {} 0 R /MediaBox [0 0 {} {}]",
                pages_obj, page.width, page.height
            );

            // Add resources if any
            if !page.resources.is_empty() {
                page_dict.push_str(" /Resources <<");
                for (key, value) in &page.resources {
                    page_dict.push_str(&format!(" /{key} {value}"));
                }
                page_dict.push_str(" >>");
            }

            // Add content stream reference if there's content
            if !page.content_stream.is_empty() {
                let content_obj = pages_obj + 1 + self.pages.len() as u32 + i as u32;
                page_dict.push_str(&format!(" /Contents {content_obj} 0 R"));
            }

            page_dict.push_str(" >>");

            pdf.extend_from_slice(format!("{object_num} 0 obj\n{page_dict}\nendobj\n").as_bytes());
            object_num += 1;
        }

        // Content streams
        for page in &self.pages {
            if !page.content_stream.is_empty() {
                xref_positions.push(pdf.len());

                let content = if self.compress_streams {
                    // TODO: Implement compression
                    page.content_stream.clone()
                } else {
                    page.content_stream.clone()
                };

                pdf.extend_from_slice(
                    format!(
                        "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
                        object_num,
                        content.len(),
                        content
                    )
                    .as_bytes(),
                );
                object_num += 1;
            }
        }

        // Info dictionary
        if !self.info.is_empty() {
            info_obj = object_num;
            xref_positions.push(pdf.len());

            let mut info_dict = "<< ".to_string();
            for (key, value) in &self.info {
                info_dict.push_str(&format!("/{} ({}) ", key, escape_pdf_string(value)));
            }
            info_dict.push_str(">>");

            pdf.extend_from_slice(format!("{info_obj} 0 obj\n{info_dict}\nendobj\n").as_bytes());
            object_num += 1;
        }

        // Additional objects
        for obj in &self.objects {
            xref_positions.push(pdf.len());
            pdf.extend_from_slice(
                format!(
                    "{} {} obj\n{}\nendobj\n",
                    obj.number, obj.generation, obj.content
                )
                .as_bytes(),
            );
        }

        // Cross-reference table
        let xref_offset = pdf.len();

        if self.use_xref_stream {
            // TODO: Implement xref stream
            self.write_traditional_xref(&mut pdf, &xref_positions, object_num);
        } else {
            self.write_traditional_xref(&mut pdf, &xref_positions, object_num);
        }

        // Trailer
        let mut trailer_dict = format!("<< /Size {object_num} /Root {catalog_obj} 0 R");
        if info_obj > 0 {
            trailer_dict.push_str(&format!(" /Info {info_obj} 0 R"));
        }
        trailer_dict.push_str(" >>");

        pdf.extend_from_slice(
            format!("trailer\n{trailer_dict}\nstartxref\n{xref_offset}\n%%EOF").as_bytes(),
        );

        pdf
    }

    /// Write traditional cross-reference table
    fn write_traditional_xref(&self, pdf: &mut Vec<u8>, positions: &[usize], num_objects: u32) {
        pdf.extend_from_slice(b"xref\n");
        pdf.extend_from_slice(format!("0 {num_objects}\n").as_bytes());

        // Entry for object 0 (always free)
        pdf.extend_from_slice(b"0000000000 65535 f \n");

        // Entries for actual objects
        for &pos in positions {
            pdf.extend_from_slice(format!("{pos:010} 00000 n \n").as_bytes());
        }
    }
}

/// Escape special characters in PDF strings
fn escape_pdf_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '(' => "\\(".to_string(),
            ')' => "\\)".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

impl Default for TestPdfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_pdf_generation() {
        let pdf = TestPdfBuilder::minimal().build();
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF"));
    }

    #[test]
    fn test_pdf_with_info() {
        let pdf = TestPdfBuilder::minimal()
            .with_title("Test PDF")
            .with_author("oxidizePdf Test Suite")
            .build();

        let pdf_str = String::from_utf8_lossy(&pdf);
        assert!(pdf_str.contains("(Test PDF)"));
        assert!(pdf_str.contains("(oxidizePdf Test Suite)"));
    }
}
