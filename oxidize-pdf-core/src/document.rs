use crate::error::Result;
use crate::objects::{Object, ObjectId};
use crate::page::Page;
use crate::writer::PdfWriter;
use std::collections::HashMap;

/// A PDF document that can contain multiple pages and metadata.
///
/// # Example
///
/// ```rust
/// use oxidize_pdf::{Document, Page};
///
/// let mut doc = Document::new();
/// doc.set_title("My Document");
/// doc.set_author("John Doe");
///
/// let page = Page::a4();
/// doc.add_page(page);
///
/// doc.save("output.pdf").unwrap();
/// ```
pub struct Document {
    pub(crate) pages: Vec<Page>,
    #[allow(dead_code)]
    pub(crate) objects: HashMap<ObjectId, Object>,
    #[allow(dead_code)]
    pub(crate) next_object_id: u32,
    pub(crate) metadata: DocumentMetadata,
}

/// Metadata for a PDF document.
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Document keywords
    pub keywords: Option<String>,
    /// Software that created the original document
    pub creator: Option<String>,
    /// Software that produced the PDF
    pub producer: Option<String>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            subject: None,
            keywords: None,
            creator: Some("oxidize_pdf".to_string()),
            producer: Some("oxidize_pdf".to_string()),
        }
    }
}

impl Document {
    /// Creates a new empty PDF document.
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            objects: HashMap::new(),
            next_object_id: 1,
            metadata: DocumentMetadata::default(),
        }
    }

    /// Adds a page to the document.
    pub fn add_page(&mut self, page: Page) {
        self.pages.push(page);
    }

    /// Sets the document title.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.metadata.title = Some(title.into());
    }

    /// Sets the document author.
    pub fn set_author(&mut self, author: impl Into<String>) {
        self.metadata.author = Some(author.into());
    }

    /// Sets the document subject.
    pub fn set_subject(&mut self, subject: impl Into<String>) {
        self.metadata.subject = Some(subject.into());
    }

    /// Sets the document keywords.
    pub fn set_keywords(&mut self, keywords: impl Into<String>) {
        self.metadata.keywords = Some(keywords.into());
    }

    /// Saves the document to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written.
    pub fn save(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let mut writer = PdfWriter::new(path)?;
        writer.write_document(self)?;
        Ok(())
    }

    /// Writes the document to a buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the PDF cannot be generated.
    pub fn write(&mut self, buffer: &mut Vec<u8>) -> Result<()> {
        let mut writer = PdfWriter::new_with_writer(buffer);
        writer.write_document(self)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn allocate_object_id(&mut self) -> ObjectId {
        let id = ObjectId::new(self.next_object_id, 0);
        self.next_object_id += 1;
        id
    }

    #[allow(dead_code)]
    pub(crate) fn add_object(&mut self, obj: Object) -> ObjectId {
        let id = self.allocate_object_id();
        self.objects.insert(id, obj);
        id
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
