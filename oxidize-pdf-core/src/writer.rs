use crate::document::Document;
use crate::error::Result;
use crate::objects::{Dictionary, Object, ObjectId};
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct PdfWriter<W: Write> {
    writer: W,
    xref_positions: HashMap<ObjectId, u64>,
    current_position: u64,
}

impl<W: Write> PdfWriter<W> {
    pub fn new_with_writer(writer: W) -> Self {
        Self {
            writer,
            xref_positions: HashMap::new(),
            current_position: 0,
        }
    }
    
    pub fn write_document(&mut self, document: &mut Document) -> Result<()> {
        self.write_header()?;
        
        // Write all objects and collect their positions
        let catalog_id = self.write_catalog()?;
        let _pages_id = self.write_pages(document)?;
        let info_id = self.write_info(document)?;
        
        // Write xref table
        let xref_position = self.current_position;
        self.write_xref()?;
        
        // Write trailer
        self.write_trailer(catalog_id, info_id, xref_position)?;
        
        if let Ok(()) = self.writer.flush() {
            // Flush succeeded
        }
        Ok(())
    }
    
    fn write_header(&mut self) -> Result<()> {
        self.write_bytes(b"%PDF-1.7\n")?;
        // Binary comment to ensure file is treated as binary
        self.write_bytes(&[b'%', 0xE2, 0xE3, 0xCF, 0xD3, b'\n'])?;
        Ok(())
    }
    
    fn write_catalog(&mut self) -> Result<ObjectId> {
        let catalog_id = ObjectId::new(1, 0);
        let pages_id = ObjectId::new(2, 0);
        
        let mut catalog = Dictionary::new();
        catalog.set("Type", Object::Name("Catalog".to_string()));
        catalog.set("Pages", Object::Reference(pages_id));
        
        self.write_object(catalog_id, Object::Dictionary(catalog))?;
        Ok(catalog_id)
    }
    
    fn write_pages(&mut self, document: &Document) -> Result<ObjectId> {
        let pages_id = ObjectId::new(2, 0);
        let mut pages_dict = Dictionary::new();
        pages_dict.set("Type", Object::Name("Pages".to_string()));
        pages_dict.set("Count", Object::Integer(document.pages.len() as i64));
        
        let mut kids = Vec::new();
        let next_id = 3;
        
        for (i, _page) in document.pages.iter().enumerate() {
            let page_id = ObjectId::new(next_id + i as u32 * 2, 0);
            kids.push(Object::Reference(page_id));
        }
        
        pages_dict.set("Kids", Object::Array(kids));
        
        self.write_object(pages_id, Object::Dictionary(pages_dict))?;
        
        // Write individual pages
        for (i, page) in document.pages.iter().enumerate() {
            let page_id = ObjectId::new(next_id + i as u32 * 2, 0);
            let content_id = ObjectId::new(next_id + i as u32 * 2 + 1, 0);
            
            self.write_page(page_id, pages_id, content_id, page)?;
            self.write_page_content(content_id, page)?;
        }
        
        Ok(pages_id)
    }
    
    fn write_page(&mut self, page_id: ObjectId, parent_id: ObjectId, content_id: ObjectId, page: &crate::page::Page) -> Result<()> {
        let mut page_dict = Dictionary::new();
        page_dict.set("Type", Object::Name("Page".to_string()));
        page_dict.set("Parent", Object::Reference(parent_id));
        page_dict.set("MediaBox", Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Real(page.width()),
            Object::Real(page.height()),
        ]));
        page_dict.set("Contents", Object::Reference(content_id));
        
        // Create resources dictionary with standard fonts
        let mut resources = Dictionary::new();
        let mut font_dict = Dictionary::new();
        
        // Add standard fonts
        for font_name in &["Helvetica", "Helvetica-Bold", "Times-Roman", "Courier"] {
            let mut font_entry = Dictionary::new();
            font_entry.set("Type", Object::Name("Font".to_string()));
            font_entry.set("Subtype", Object::Name("Type1".to_string()));
            font_entry.set("BaseFont", Object::Name(font_name.to_string()));
            font_dict.set(*font_name, Object::Dictionary(font_entry));
        }
        
        resources.set("Font", Object::Dictionary(font_dict));
        
        // Add images as XObjects
        if !page.images().is_empty() {
            let mut xobject_dict = Dictionary::new();
            let mut image_id_counter = 1000; // Start high to avoid conflicts
            
            for (name, image) in page.images() {
                let image_id = ObjectId::new(image_id_counter, 0);
                image_id_counter += 1;
                
                // Write the image XObject
                self.write_object(image_id, image.to_pdf_object())?;
                
                // Add reference to XObject dictionary
                xobject_dict.set(name, Object::Reference(image_id));
            }
            
            resources.set("XObject", Object::Dictionary(xobject_dict));
        }
        
        page_dict.set("Resources", Object::Dictionary(resources));
        
        self.write_object(page_id, Object::Dictionary(page_dict))?;
        Ok(())
    }
    
    fn write_page_content(&mut self, content_id: ObjectId, page: &crate::page::Page) -> Result<()> {
        let mut page_copy = page.clone();
        let content = page_copy.generate_content()?;
        
        // Create stream with compression if enabled
        #[cfg(feature = "compression")]
        {
            use crate::objects::Stream;
            let mut stream = Stream::new(content);
            stream.compress_flate()?;
            
            self.write_object(content_id, Object::Stream(
                stream.dictionary().clone(),
                stream.data().to_vec()
            ))?;
        }
        
        #[cfg(not(feature = "compression"))]
        {
            let mut stream_dict = Dictionary::new();
            stream_dict.set("Length", Object::Integer(content.len() as i64));
            
            self.write_object(content_id, Object::Stream(stream_dict, content))?;
        }
        
        Ok(())
    }
    
    fn write_info(&mut self, document: &Document) -> Result<ObjectId> {
        let info_id = ObjectId::new(100, 0); // Use high ID to avoid conflicts
        let mut info_dict = Dictionary::new();
        
        if let Some(ref title) = document.metadata.title {
            info_dict.set("Title", Object::String(title.clone()));
        }
        if let Some(ref author) = document.metadata.author {
            info_dict.set("Author", Object::String(author.clone()));
        }
        if let Some(ref subject) = document.metadata.subject {
            info_dict.set("Subject", Object::String(subject.clone()));
        }
        if let Some(ref keywords) = document.metadata.keywords {
            info_dict.set("Keywords", Object::String(keywords.clone()));
        }
        if let Some(ref creator) = document.metadata.creator {
            info_dict.set("Creator", Object::String(creator.clone()));
        }
        if let Some(ref producer) = document.metadata.producer {
            info_dict.set("Producer", Object::String(producer.clone()));
        }
        
        // Add creation date
        let now = chrono::Local::now();
        let date_string = format!("D:{}", now.format("%Y%m%d%H%M%S%z"));
        info_dict.set("CreationDate", Object::String(date_string.clone()));
        info_dict.set("ModDate", Object::String(date_string));
        
        self.write_object(info_id, Object::Dictionary(info_dict))?;
        Ok(info_id)
    }
    
}

impl PdfWriter<BufWriter<std::fs::File>> {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::create(path)?;
        let writer = BufWriter::new(file);
        
        Ok(Self {
            writer,
            xref_positions: HashMap::new(),
            current_position: 0,
        })
    }
}

impl<W: Write> PdfWriter<W> {
    fn write_object(&mut self, id: ObjectId, object: Object) -> Result<()> {
        self.xref_positions.insert(id, self.current_position);
        
        let header = format!("{} {} obj\n", id.number(), id.generation());
        self.write_bytes(header.as_bytes())?;
        
        self.write_object_value(&object)?;
        
        self.write_bytes(b"\nendobj\n")?;
        Ok(())
    }
    
    fn write_object_value(&mut self, object: &Object) -> Result<()> {
        match object {
            Object::Null => self.write_bytes(b"null")?,
            Object::Boolean(b) => self.write_bytes(if *b { b"true" } else { b"false" })?,
            Object::Integer(i) => self.write_bytes(i.to_string().as_bytes())?,
            Object::Real(f) => self.write_bytes(format!("{:.6}", f).trim_end_matches('0').trim_end_matches('.').as_bytes())?,
            Object::String(s) => {
                self.write_bytes(b"(")?;
                self.write_bytes(s.as_bytes())?;
                self.write_bytes(b")")?;
            }
            Object::Name(n) => {
                self.write_bytes(b"/")?;
                self.write_bytes(n.as_bytes())?;
            }
            Object::Array(arr) => {
                self.write_bytes(b"[")?;
                for (i, obj) in arr.iter().enumerate() {
                    if i > 0 {
                        self.write_bytes(b" ")?;
                    }
                    self.write_object_value(obj)?;
                }
                self.write_bytes(b"]")?;
            }
            Object::Dictionary(dict) => {
                self.write_bytes(b"<<")?;
                for (key, value) in dict.entries() {
                    self.write_bytes(b"\n/")?;
                    self.write_bytes(key.as_bytes())?;
                    self.write_bytes(b" ")?;
                    self.write_object_value(value)?;
                }
                self.write_bytes(b"\n>>")?;
            }
            Object::Stream(dict, data) => {
                self.write_object_value(&Object::Dictionary(dict.clone()))?;
                self.write_bytes(b"\nstream\n")?;
                self.write_bytes(data)?;
                self.write_bytes(b"\nendstream")?;
            }
            Object::Reference(id) => {
                let ref_str = format!("{} {} R", id.number(), id.generation());
                self.write_bytes(ref_str.as_bytes())?;
            }
        }
        Ok(())
    }
    
    fn write_xref(&mut self) -> Result<()> {
        self.write_bytes(b"xref\n")?;
        
        // Sort by object number and write entries
        let mut entries: Vec<_> = self.xref_positions.iter().map(|(id, pos)| (*id, *pos)).collect();
        entries.sort_by_key(|(id, _)| id.number());
        
        // Find the highest object number to determine size
        let max_obj_num = entries.iter()
            .map(|(id, _)| id.number())
            .max()
            .unwrap_or(0);
        
        // Write subsection header - PDF 1.7 spec allows multiple subsections
        // For simplicity, write one subsection from 0 to max
        self.write_bytes(b"0 ")?;
        self.write_bytes((max_obj_num + 1).to_string().as_bytes())?;
        self.write_bytes(b"\n")?;
        
        // Write free object entry
        self.write_bytes(b"0000000000 65535 f \n")?;
        
        // Write entries for all object numbers from 1 to max
        // Fill in gaps with free entries
        for obj_num in 1..=max_obj_num {
            let obj_id = ObjectId::new(obj_num, 0);
            if let Some((_, position)) = entries.iter().find(|(id, _)| id.number() == obj_num) {
                let entry = format!("{:010} {:05} n \n", position, 0);
                self.write_bytes(entry.as_bytes())?;
            } else {
                // Free entry for gap
                self.write_bytes(b"0000000000 00000 f \n")?;
            }
        }
        
        Ok(())
    }
    
    fn write_trailer(&mut self, catalog_id: ObjectId, info_id: ObjectId, xref_position: u64) -> Result<()> {
        // Find the highest object number to determine size
        let max_obj_num = self.xref_positions.keys()
            .map(|id| id.number())
            .max()
            .unwrap_or(0);
        
        let mut trailer = Dictionary::new();
        trailer.set("Size", Object::Integer((max_obj_num + 1) as i64));
        trailer.set("Root", Object::Reference(catalog_id));
        trailer.set("Info", Object::Reference(info_id));
        
        self.write_bytes(b"trailer\n")?;
        self.write_object_value(&Object::Dictionary(trailer))?;
        self.write_bytes(b"\nstartxref\n")?;
        self.write_bytes(xref_position.to_string().as_bytes())?;
        self.write_bytes(b"\n%%EOF\n")?;
        
        Ok(())
    }
    
    fn write_bytes(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data)?;
        self.current_position += data.len() as u64;
        Ok(())
    }
}