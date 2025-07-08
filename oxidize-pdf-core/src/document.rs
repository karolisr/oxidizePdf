use crate::error::Result;
use crate::objects::{Object, ObjectId};
use crate::page::Page;
use crate::writer::PdfWriter;
use std::collections::HashMap;

pub struct Document {
    pub(crate) pages: Vec<Page>,
    pub(crate) objects: HashMap<ObjectId, Object>,
    pub(crate) next_object_id: u32,
    pub(crate) metadata: DocumentMetadata,
}

#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
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
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            objects: HashMap::new(),
            next_object_id: 1,
            metadata: DocumentMetadata::default(),
        }
    }
    
    pub fn add_page(&mut self, page: Page) {
        self.pages.push(page);
    }
    
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.metadata.title = Some(title.into());
    }
    
    pub fn set_author(&mut self, author: impl Into<String>) {
        self.metadata.author = Some(author.into());
    }
    
    pub fn save(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let mut writer = PdfWriter::new(path)?;
        writer.write_document(self)?;
        Ok(())
    }
    
    pub(crate) fn allocate_object_id(&mut self) -> ObjectId {
        let id = ObjectId::new(self.next_object_id, 0);
        self.next_object_id += 1;
        id
    }
    
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