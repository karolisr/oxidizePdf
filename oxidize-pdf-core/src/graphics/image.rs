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
    /// PNG format
    Png,
    /// TIFF format
    Tiff,
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

    /// Load a PNG image from a file
    pub fn from_png_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Self::from_png_data(data)
    }

    /// Create an image from PNG data
    pub fn from_png_data(data: Vec<u8>) -> Result<Self> {
        // Parse PNG header to get dimensions and color info
        let (width, height, color_space, bits_per_component) = parse_png_header(&data)?;

        Ok(Image {
            data,
            format: ImageFormat::Png,
            width,
            height,
            color_space,
            bits_per_component,
        })
    }

    /// Load a TIFF image from a file
    pub fn from_tiff_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Self::from_tiff_data(data)
    }

    /// Create an image from TIFF data
    pub fn from_tiff_data(data: Vec<u8>) -> Result<Self> {
        // Parse TIFF header to get dimensions and color info
        let (width, height, color_space, bits_per_component) = parse_tiff_header(&data)?;

        Ok(Image {
            data,
            format: ImageFormat::Tiff,
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

    /// Get image format
    pub fn format(&self) -> ImageFormat {
        self.format
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

        // Filter based on image format
        match self.format {
            ImageFormat::Jpeg => {
                dict.set("Filter", Object::Name("DCTDecode".to_string()));
            }
            ImageFormat::Png => {
                dict.set("Filter", Object::Name("FlateDecode".to_string()));
            }
            ImageFormat::Tiff => {
                // TIFF can use various filters, but commonly LZW or FlateDecode
                dict.set("Filter", Object::Name("FlateDecode".to_string()));
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

/// Parse PNG header to extract image information
fn parse_png_header(data: &[u8]) -> Result<(u32, u32, ColorSpace, u8)> {
    // PNG signature: 8 bytes
    if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(PdfError::InvalidImage("Not a valid PNG file".to_string()));
    }

    // Find IHDR chunk (should be first chunk after signature)
    let mut pos = 8;

    while pos + 8 < data.len() {
        // Read chunk length (4 bytes, big-endian)
        let chunk_length =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;

        // Read chunk type (4 bytes)
        let chunk_type = &data[pos + 4..pos + 8];

        if chunk_type == b"IHDR" {
            // IHDR chunk found
            if pos + 8 + chunk_length > data.len() || chunk_length < 13 {
                return Err(PdfError::InvalidImage("Invalid PNG IHDR chunk".to_string()));
            }

            let ihdr_data = &data[pos + 8..pos + 8 + chunk_length];

            // Parse IHDR data
            let width =
                u32::from_be_bytes([ihdr_data[0], ihdr_data[1], ihdr_data[2], ihdr_data[3]]);

            let height =
                u32::from_be_bytes([ihdr_data[4], ihdr_data[5], ihdr_data[6], ihdr_data[7]]);

            let bit_depth = ihdr_data[8];
            let color_type = ihdr_data[9];

            // Map PNG color types to PDF color spaces
            let color_space = match color_type {
                0 => ColorSpace::DeviceGray, // Grayscale
                2 => ColorSpace::DeviceRGB,  // RGB
                3 => ColorSpace::DeviceRGB,  // Palette (treated as RGB)
                4 => ColorSpace::DeviceGray, // Grayscale + Alpha
                6 => ColorSpace::DeviceRGB,  // RGB + Alpha
                _ => {
                    return Err(PdfError::InvalidImage(format!(
                        "Unsupported PNG color type: {color_type}"
                    )))
                }
            };

            return Ok((width, height, color_space, bit_depth));
        }

        // Skip to next chunk
        pos += 8 + chunk_length + 4; // header + data + CRC
    }

    Err(PdfError::InvalidImage(
        "PNG IHDR chunk not found".to_string(),
    ))
}

/// Parse TIFF header to extract image information
fn parse_tiff_header(data: &[u8]) -> Result<(u32, u32, ColorSpace, u8)> {
    if data.len() < 8 {
        return Err(PdfError::InvalidImage(
            "Invalid TIFF file: too short".to_string(),
        ));
    }

    // Check byte order (first 2 bytes)
    let (is_little_endian, offset) = if &data[0..2] == b"II" {
        (true, 2) // Little endian
    } else if &data[0..2] == b"MM" {
        (false, 2) // Big endian
    } else {
        return Err(PdfError::InvalidImage(
            "Invalid TIFF byte order".to_string(),
        ));
    };

    // Check magic number (should be 42)
    let magic = if is_little_endian {
        u16::from_le_bytes([data[offset], data[offset + 1]])
    } else {
        u16::from_be_bytes([data[offset], data[offset + 1]])
    };

    if magic != 42 {
        return Err(PdfError::InvalidImage(
            "Invalid TIFF magic number".to_string(),
        ));
    }

    // Get offset to first IFD (Image File Directory)
    let ifd_offset = if is_little_endian {
        u32::from_le_bytes([
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
        ])
    } else {
        u32::from_be_bytes([
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
        ])
    } as usize;

    if ifd_offset + 2 > data.len() {
        return Err(PdfError::InvalidImage(
            "Invalid TIFF IFD offset".to_string(),
        ));
    }

    // Read number of directory entries
    let num_entries = if is_little_endian {
        u16::from_le_bytes([data[ifd_offset], data[ifd_offset + 1]])
    } else {
        u16::from_be_bytes([data[ifd_offset], data[ifd_offset + 1]])
    };

    let mut width = 0u32;
    let mut height = 0u32;
    let mut bits_per_sample = 8u16;
    let mut photometric_interpretation = 0u16;

    // Read directory entries
    for i in 0..num_entries {
        let entry_offset = ifd_offset + 2 + (i as usize * 12);

        if entry_offset + 12 > data.len() {
            break;
        }

        let tag = if is_little_endian {
            u16::from_le_bytes([data[entry_offset], data[entry_offset + 1]])
        } else {
            u16::from_be_bytes([data[entry_offset], data[entry_offset + 1]])
        };

        let value_offset = entry_offset + 8;

        match tag {
            256 => {
                // ImageWidth
                width = if is_little_endian {
                    u32::from_le_bytes([
                        data[value_offset],
                        data[value_offset + 1],
                        data[value_offset + 2],
                        data[value_offset + 3],
                    ])
                } else {
                    u32::from_be_bytes([
                        data[value_offset],
                        data[value_offset + 1],
                        data[value_offset + 2],
                        data[value_offset + 3],
                    ])
                };
            }
            257 => {
                // ImageHeight
                height = if is_little_endian {
                    u32::from_le_bytes([
                        data[value_offset],
                        data[value_offset + 1],
                        data[value_offset + 2],
                        data[value_offset + 3],
                    ])
                } else {
                    u32::from_be_bytes([
                        data[value_offset],
                        data[value_offset + 1],
                        data[value_offset + 2],
                        data[value_offset + 3],
                    ])
                };
            }
            258 => {
                // BitsPerSample
                bits_per_sample = if is_little_endian {
                    u16::from_le_bytes([data[value_offset], data[value_offset + 1]])
                } else {
                    u16::from_be_bytes([data[value_offset], data[value_offset + 1]])
                };
            }
            262 => {
                // PhotometricInterpretation
                photometric_interpretation = if is_little_endian {
                    u16::from_le_bytes([data[value_offset], data[value_offset + 1]])
                } else {
                    u16::from_be_bytes([data[value_offset], data[value_offset + 1]])
                };
            }
            _ => {} // Skip unknown tags
        }
    }

    if width == 0 || height == 0 {
        return Err(PdfError::InvalidImage(
            "TIFF dimensions not found".to_string(),
        ));
    }

    // Map TIFF photometric interpretation to PDF color space
    let color_space = match photometric_interpretation {
        0 | 1 => ColorSpace::DeviceGray, // White is zero | Black is zero
        2 => ColorSpace::DeviceRGB,      // RGB
        5 => ColorSpace::DeviceCMYK,     // CMYK
        _ => ColorSpace::DeviceRGB,      // Default to RGB
    };

    Ok((width, height, color_space, bits_per_sample as u8))
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

    #[test]
    fn test_parse_png_header() {
        // Minimal PNG header for testing
        let mut png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR chunk length (13)
            0x49, 0x48, 0x44, 0x52, // IHDR chunk type
            0x00, 0x00, 0x00, 0x64, // Width (100)
            0x00, 0x00, 0x00, 0x64, // Height (100)
            0x08, // Bit depth (8)
            0x02, // Color type (2 = RGB)
            0x00, // Compression method
            0x00, // Filter method
            0x00, // Interlace method
        ];

        // Add CRC (simplified - just 4 bytes)
        png_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let result = parse_png_header(&png_data);
        assert!(result.is_ok());
        let (width, height, color_space, bits) = result.unwrap();
        assert_eq!(width, 100);
        assert_eq!(height, 100);
        assert_eq!(color_space, ColorSpace::DeviceRGB);
        assert_eq!(bits, 8);
    }

    #[test]
    fn test_invalid_png() {
        let invalid_data = vec![0x00, 0x00];
        let result = parse_png_header(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_tiff_header_little_endian() {
        // Minimal TIFF header for testing (little endian)
        let tiff_data = vec![
            0x49, 0x49, // Little endian byte order
            0x2A, 0x00, // Magic number (42)
            0x08, 0x00, 0x00, 0x00, // Offset to first IFD
            0x03, 0x00, // Number of directory entries
            // ImageWidth tag (256)
            0x00, 0x01, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00,
            // ImageHeight tag (257)
            0x01, 0x01, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00,
            // BitsPerSample tag (258)
            0x02, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, // Next IFD offset (0 = none)
        ];

        let result = parse_tiff_header(&tiff_data);
        assert!(result.is_ok());
        let (width, height, color_space, bits) = result.unwrap();
        assert_eq!(width, 100);
        assert_eq!(height, 100);
        assert_eq!(color_space, ColorSpace::DeviceGray);
        assert_eq!(bits, 8);
    }

    #[test]
    fn test_parse_tiff_header_big_endian() {
        // Minimal TIFF header for testing (big endian)
        let tiff_data = vec![
            0x4D, 0x4D, // Big endian byte order
            0x00, 0x2A, // Magic number (42)
            0x00, 0x00, 0x00, 0x08, // Offset to first IFD
            0x00, 0x03, // Number of directory entries
            // ImageWidth tag (256)
            0x01, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x64,
            // ImageHeight tag (257)
            0x01, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x64,
            // BitsPerSample tag (258)
            0x01, 0x02, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, // Next IFD offset (0 = none)
        ];

        let result = parse_tiff_header(&tiff_data);
        assert!(result.is_ok());
        let (width, height, color_space, bits) = result.unwrap();
        assert_eq!(width, 100);
        assert_eq!(height, 100);
        assert_eq!(color_space, ColorSpace::DeviceGray);
        assert_eq!(bits, 8);
    }

    #[test]
    fn test_invalid_tiff() {
        let invalid_data = vec![0x00, 0x00];
        let result = parse_tiff_header(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_image_format_enum() {
        assert_eq!(ImageFormat::Jpeg, ImageFormat::Jpeg);
        assert_eq!(ImageFormat::Png, ImageFormat::Png);
        assert_eq!(ImageFormat::Tiff, ImageFormat::Tiff);
        assert_ne!(ImageFormat::Jpeg, ImageFormat::Png);
    }
}
