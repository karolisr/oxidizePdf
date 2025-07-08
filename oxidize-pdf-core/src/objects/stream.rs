use crate::objects::Dictionary;
#[cfg(feature = "compression")]
use crate::error::{PdfError, Result};

#[derive(Debug, Clone)]
pub struct Stream {
    dictionary: Dictionary,
    data: Vec<u8>,
}

impl Stream {
    pub fn new(data: Vec<u8>) -> Self {
        let mut dictionary = Dictionary::new();
        dictionary.set("Length", data.len() as i64);
        
        Self { dictionary, data }
    }
    
    pub fn with_dictionary(dictionary: Dictionary, data: Vec<u8>) -> Self {
        let mut dict = dictionary;
        dict.set("Length", data.len() as i64);
        
        Self {
            dictionary: dict,
            data,
        }
    }
    
    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }
    
    pub fn dictionary_mut(&mut self) -> &mut Dictionary {
        &mut self.dictionary
    }
    
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
    
    pub fn set_filter(&mut self, filter: &str) {
        self.dictionary.set("Filter", format!("/{}", filter));
    }
    
    pub fn set_decode_params(&mut self, params: Dictionary) {
        self.dictionary.set("DecodeParms", params);
    }
    
    #[cfg(feature = "compression")]
    pub fn compress_flate(&mut self) -> Result<()> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.data)
            .map_err(|e| PdfError::CompressionError(e.to_string()))?;
        let compressed = encoder.finish()
            .map_err(|e| PdfError::CompressionError(e.to_string()))?;
        
        self.data = compressed;
        self.dictionary.set("Length", self.data.len() as i64);
        self.set_filter("FlateDecode");
        
        Ok(())
    }
}