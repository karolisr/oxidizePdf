//! Image support for PDF generation
//!
//! Currently supports:
//! - JPEG images

use crate::objects::{Dictionary, Object};
use crate::{PdfError, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Represents an image that can be embedded in a PDF
#[derive(Debug, Clone)]
pub struct Image {
    /// Image data
    data: Vec<u8>,
    /// Image format
    format: ImageFormat,
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
    /// Color space
    color_space: ColorSpace,
    /// Bits per component
    bits_per_component: u8,
}

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    /// JPEG format
    Jpeg,
    // Future: PNG, TIFF, etc.
}

/// Color spaces for images
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSpace {
    /// Grayscale
    DeviceGray,
    /// RGB color
    DeviceRGB,
    /// CMYK color
    DeviceCMYK,
}

impl Image {
    /// Load a JPEG image from a file
    pub fn from_jpeg_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Self::from_jpeg_data(data)
    }

    /// Create an image from JPEG data
    pub fn from_jpeg_data(data: Vec<u8>) -> Result<Self> {
        // Parse JPEG header to get dimensions and color info
        let (width, height, color_space, bits_per_component) = parse_jpeg_header(&data)?;

        Ok(Image {
            data,
            format: ImageFormat::Jpeg,
            width,
            height,
            color_space,
            bits_per_component,
        })
    }

    /// Get image width in pixels
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get image height in pixels
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get image data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Convert to PDF XObject
    pub fn to_pdf_object(&self) -> Object {
        let mut dict = Dictionary::new();

        // Required entries for image XObject
        dict.set("Type", Object::Name("XObject".to_string()));
        dict.set("Subtype", Object::Name("Image".to_string()));
        dict.set("Width", Object::Integer(self.width as i64));
        dict.set("Height", Object::Integer(self.height as i64));

        // Color space
        let color_space_name = match self.color_space {
            ColorSpace::DeviceGray => "DeviceGray",
            ColorSpace::DeviceRGB => "DeviceRGB",
            ColorSpace::DeviceCMYK => "DeviceCMYK",
        };
        dict.set("ColorSpace", Object::Name(color_space_name.to_string()));

        // Bits per component
        dict.set(
            "BitsPerComponent",
            Object::Integer(self.bits_per_component as i64),
        );

        // Filter for JPEG
        match self.format {
            ImageFormat::Jpeg => {
                dict.set("Filter", Object::Name("DCTDecode".to_string()));
            }
        }

        // Create stream with image data
        Object::Stream(dict, self.data.clone())
    }
}

/// Parse JPEG header to extract image information
fn parse_jpeg_header(data: &[u8]) -> Result<(u32, u32, ColorSpace, u8)> {
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err(PdfError::InvalidImage("Not a valid JPEG file".to_string()));
    }

    let mut pos = 2;
    let mut width = 0;
    let mut height = 0;
    let mut components = 0;

    while pos < data.len() - 1 {
        if data[pos] != 0xFF {
            return Err(PdfError::InvalidImage("Invalid JPEG marker".to_string()));
        }

        let marker = data[pos + 1];
        pos += 2;

        // Skip padding bytes
        if marker == 0xFF {
            continue;
        }

        // Check for SOF markers (Start of Frame)
        if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
            // This is a SOF marker
            if pos + 7 >= data.len() {
                return Err(PdfError::InvalidImage("Truncated JPEG file".to_string()));
            }

            // Skip length
            pos += 2;

            // Skip precision
            pos += 1;

            // Read height and width
            height = ((data[pos] as u32) << 8) | (data[pos + 1] as u32);
            pos += 2;
            width = ((data[pos] as u32) << 8) | (data[pos + 1] as u32);
            pos += 2;

            // Read number of components
            components = data[pos];
            break;
        } else if marker == 0xD9 {
            // End of image
            break;
        } else if marker == 0xD8 || (0xD0..=0xD7).contains(&marker) {
            // No length field for these markers
            continue;
        } else {
            // Read length and skip segment
            if pos + 1 >= data.len() {
                return Err(PdfError::InvalidImage("Truncated JPEG file".to_string()));
            }
            let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
            pos += length;
        }
    }

    if width == 0 || height == 0 {
        return Err(PdfError::InvalidImage(
            "Could not find image dimensions".to_string(),
        ));
    }

    let color_space = match components {
        1 => ColorSpace::DeviceGray,
        3 => ColorSpace::DeviceRGB,
        4 => ColorSpace::DeviceCMYK,
        _ => {
            return Err(PdfError::InvalidImage(format!(
                "Unsupported number of components: {components}"
            )))
        }
    };

    Ok((width, height, color_space, 8)) // JPEG typically uses 8 bits per component
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_jpeg_header() {
        // Minimal JPEG header for testing
        let jpeg_data = vec![
            0xFF, 0xD8, // SOI marker
            0xFF, 0xC0, // SOF0 marker
            0x00, 0x11, // Length (17 bytes)
            0x08, // Precision (8 bits)
            0x00, 0x64, // Height (100)
            0x00, 0xC8, // Width (200)
            0x03, // Components (3 = RGB)
                  // ... rest of data
        ];

        let result = parse_jpeg_header(&jpeg_data);
        assert!(result.is_ok());
        let (width, height, color_space, bits) = result.unwrap();
        assert_eq!(width, 200);
        assert_eq!(height, 100);
        assert_eq!(color_space, ColorSpace::DeviceRGB);
        assert_eq!(bits, 8);
    }

    #[test]
    fn test_invalid_jpeg() {
        let invalid_data = vec![0x00, 0x00];
        let result = parse_jpeg_header(&invalid_data);
        assert!(result.is_err());
    }
}
