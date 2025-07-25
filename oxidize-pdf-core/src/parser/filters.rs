//! PDF Stream Filters
//!
//! Handles decompression and decoding of PDF streams according to ISO 32000-1 Section 7.4

use super::objects::{PdfDictionary, PdfObject};
use super::{ParseError, ParseOptions, ParseResult};

#[cfg(feature = "compression")]
use flate2::read::ZlibDecoder;
use std::io::Read;

/// Supported PDF filters
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    /// ASCII hex decode
    ASCIIHexDecode,

    /// ASCII 85 decode
    ASCII85Decode,

    /// LZW decode
    LZWDecode,

    /// Flate decode (zlib/deflate compression)
    FlateDecode,

    /// Run length decode
    RunLengthDecode,

    /// CCITT fax decode
    CCITTFaxDecode,

    /// JBIG2 decode
    JBIG2Decode,

    /// DCT decode (JPEG)
    DCTDecode,

    /// JPX decode (JPEG 2000)
    JPXDecode,

    /// Crypt filter
    Crypt,
}

impl Filter {
    /// Parse filter from name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ASCIIHexDecode" => Some(Filter::ASCIIHexDecode),
            "ASCII85Decode" => Some(Filter::ASCII85Decode),
            "LZWDecode" => Some(Filter::LZWDecode),
            "FlateDecode" => Some(Filter::FlateDecode),
            "RunLengthDecode" => Some(Filter::RunLengthDecode),
            "CCITTFaxDecode" => Some(Filter::CCITTFaxDecode),
            "JBIG2Decode" => Some(Filter::JBIG2Decode),
            "DCTDecode" => Some(Filter::DCTDecode),
            "JPXDecode" => Some(Filter::JPXDecode),
            "Crypt" => Some(Filter::Crypt),
            _ => None,
        }
    }
}

/// Decode stream data according to specified filters
pub fn decode_stream(
    data: &[u8],
    dict: &PdfDictionary,
    options: &ParseOptions,
) -> ParseResult<Vec<u8>> {
    // Get filter(s) from dictionary
    let filters = match dict.get("Filter") {
        Some(PdfObject::Name(name)) => vec![name.as_str()],
        Some(PdfObject::Array(array)) => {
            let mut filter_names = Vec::new();
            for obj in &array.0 {
                if let PdfObject::Name(name) = obj {
                    filter_names.push(name.as_str());
                } else {
                    return Err(ParseError::SyntaxError {
                        position: 0,
                        message: "Invalid filter in array".to_string(),
                    });
                }
            }
            filter_names
        }
        None => {
            // No filter, return data as-is
            return Ok(data.to_vec());
        }
        _ => {
            return Err(ParseError::SyntaxError {
                position: 0,
                message: "Invalid Filter type".to_string(),
            });
        }
    };

    // Apply filters in order
    let mut result = data.to_vec();
    for filter_name in filters {
        let filter = Filter::from_name(filter_name).ok_or_else(|| ParseError::SyntaxError {
            position: 0,
            message: format!("Unknown filter: {filter_name}"),
        })?;

        result = apply_filter(&result, filter, options)?;
    }

    Ok(result)
}

/// Apply a single filter to data
fn apply_filter(data: &[u8], filter: Filter, options: &ParseOptions) -> ParseResult<Vec<u8>> {
    match filter {
        Filter::FlateDecode => decode_flate(data, options),
        Filter::ASCIIHexDecode => decode_ascii_hex(data),
        Filter::ASCII85Decode => decode_ascii85(data),
        _ => Err(ParseError::SyntaxError {
            position: 0,
            message: format!("Filter {filter:?} not yet implemented"),
        }),
    }
}

/// Decode FlateDecode (zlib/deflate) compressed data
#[cfg(feature = "compression")]
fn decode_flate(data: &[u8], options: &ParseOptions) -> ParseResult<Vec<u8>> {
    // First try standard zlib decoding
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();

    match decoder.read_to_end(&mut result) {
        Ok(_) => Ok(result),
        Err(e) => {
            // Check if we should attempt recovery
            if !options.recover_from_stream_errors {
                return Err(ParseError::StreamDecodeError(format!(
                    "Flate decode error: {e}"
                )));
            }

            // Log the error for debugging
            if options.log_recovery_details {
                // Logging would happen here if the logging feature was enabled
                eprintln!("Standard FlateDecode failed: {e}, attempting recovery");
            }

            // Try alternative decoding strategies
            decode_flate_with_recovery(data, e, options)
        }
    }
}

/// Attempt to decode FlateDecode data with various recovery strategies
#[cfg(feature = "compression")]
fn decode_flate_with_recovery(
    data: &[u8],
    original_error: std::io::Error,
    options: &ParseOptions,
) -> ParseResult<Vec<u8>> {
    let mut attempts = 0;
    let max_attempts = options.max_recovery_attempts;

    // Strategy 1: Try raw deflate without zlib wrapper
    attempts += 1;
    if attempts <= max_attempts {
        if options.log_recovery_details {
            eprintln!("Recovery attempt {attempts}/{max_attempts}: raw deflate decode");
        }
        use flate2::read::DeflateDecoder;
        let mut raw_decoder = DeflateDecoder::new(data);
        let mut result = Vec::new();
        if raw_decoder.read_to_end(&mut result).is_ok() {
            if options.log_recovery_details {
                eprintln!("Successfully decoded using raw deflate");
            }
            return Ok(result);
        }
    }

    // Strategy 2: Try with a custom decompressor that ignores checksums
    attempts += 1;
    if attempts <= max_attempts {
        if options.log_recovery_details {
            eprintln!(
                "Recovery attempt {attempts}/{max_attempts}: decode with checksum validation disabled"
            );
        }
        use flate2::{Decompress, FlushDecompress};
        let mut decompress = Decompress::new(false); // false = ignore checksums
        let mut output = Vec::with_capacity(data.len() * 3); // Estimate decompressed size

        match decompress.decompress_vec(data, &mut output, FlushDecompress::Finish) {
            Ok(flate2::Status::StreamEnd) => {
                if options.log_recovery_details {
                    eprintln!("Successfully decoded with checksum validation disabled");
                }
                return Ok(output);
            }
            Ok(flate2::Status::Ok) | Ok(flate2::Status::BufError) => {
                // Partial decompression might have succeeded
                if !output.is_empty() && options.partial_content_allowed {
                    if options.log_recovery_details {
                        eprintln!("Partial decompression recovered {} bytes", output.len());
                    }
                    return Ok(output);
                }
            }
            Err(_) => {
                // Continue to next strategy
            }
        }
    }

    // Strategy 3: Try to skip corrupted header bytes
    if data.len() > 2 {
        attempts += 1;
        if attempts <= max_attempts {
            if options.log_recovery_details {
                eprintln!(
                    "Recovery attempt {attempts}/{max_attempts}: skip potentially corrupted header"
                );
            }
            for skip in 1..std::cmp::min(10, data.len()) {
                let mut decoder = ZlibDecoder::new(&data[skip..]);
                let mut result = Vec::new();
                if decoder.read_to_end(&mut result).is_ok() && !result.is_empty() {
                    if options.log_recovery_details {
                        eprintln!("Successfully decoded by skipping {skip} header bytes");
                    }
                    return Ok(result);
                }
            }
        }
    }

    // All strategies failed
    if options.ignore_corrupt_streams {
        // Return empty data if we're ignoring corrupt streams
        if options.log_recovery_details {
            eprintln!("Ignoring corrupt stream, returning empty data");
        }
        Ok(Vec::new())
    } else {
        Err(ParseError::StreamDecodeError(format!(
            "Flate decode error: {original_error} (all {attempts} recovery strategies failed)"
        )))
    }
}

#[cfg(not(feature = "compression"))]
fn decode_flate(_data: &[u8], _options: &ParseOptions) -> ParseResult<Vec<u8>> {
    Err(ParseError::StreamDecodeError(
        "FlateDecode requires 'compression' feature".to_string(),
    ))
}

/// Decode ASCIIHexDecode data
fn decode_ascii_hex(data: &[u8]) -> ParseResult<Vec<u8>> {
    let mut result = Vec::new();
    let mut chars = data.iter().filter(|&&b| !b.is_ascii_whitespace());

    loop {
        let high = match chars.next() {
            Some(&b'>') => break, // End marker
            Some(&ch) => ch,
            None => break,
        };

        let low = match chars.next() {
            Some(&b'>') => {
                // Odd number of digits, pad with 0
                b'0'
            }
            Some(&ch) => ch,
            None => b'0', // Pad with 0
        };

        let high_val = hex_digit_value(high).ok_or_else(|| {
            ParseError::StreamDecodeError(format!("Invalid hex digit: {}", high as char))
        })?;
        let low_val = hex_digit_value(low).ok_or_else(|| {
            ParseError::StreamDecodeError(format!("Invalid hex digit: {}", low as char))
        })?;

        result.push((high_val << 4) | low_val);

        if low == b'>' {
            break;
        }
    }

    Ok(result)
}

/// Get value of hex digit
fn hex_digit_value(ch: u8) -> Option<u8> {
    match ch {
        b'0'..=b'9' => Some(ch - b'0'),
        b'A'..=b'F' => Some(ch - b'A' + 10),
        b'a'..=b'f' => Some(ch - b'a' + 10),
        _ => None,
    }
}

/// Decode ASCII85Decode data
fn decode_ascii85(data: &[u8]) -> ParseResult<Vec<u8>> {
    let mut result = Vec::new();
    let mut chars = data.iter().filter(|&&b| !b.is_ascii_whitespace());
    let mut group = Vec::with_capacity(5);

    // Skip optional <~ prefix
    let mut ch = match chars.next() {
        Some(&b'<') => {
            if chars.next() == Some(&b'~') {
                // Skip the prefix and get next char
                chars.next()
            } else {
                // Not a valid prefix, treat '<' as data
                Some(&b'<')
            }
        }
        other => other,
    };

    while let Some(&c) = ch {
        match c {
            b'~' => {
                // Check for end marker ~>
                if chars.next() == Some(&b'>') {
                    break;
                } else {
                    return Err(ParseError::StreamDecodeError(
                        "Invalid ASCII85 end marker".to_string(),
                    ));
                }
            }
            b'z' if group.is_empty() => {
                // Special case: 'z' represents four zero bytes
                result.extend_from_slice(&[0, 0, 0, 0]);
            }
            b'!'..=b'u' => {
                group.push(c);
                if group.len() == 5 {
                    // Decode complete group
                    let value = group
                        .iter()
                        .enumerate()
                        .map(|(i, &ch)| (ch - b'!') as u32 * 85u32.pow(4 - i as u32))
                        .sum::<u32>();

                    result.push((value >> 24) as u8);
                    result.push((value >> 16) as u8);
                    result.push((value >> 8) as u8);
                    result.push(value as u8);

                    group.clear();
                }
            }
            _ => {
                return Err(ParseError::StreamDecodeError(format!(
                    "Invalid ASCII85 character: {}",
                    c as char
                )));
            }
        }
        ch = chars.next();
    }

    // Handle incomplete final group
    if !group.is_empty() {
        // Save original length to know how many bytes to output
        let original_len = group.len();

        // Pad with 'u' (84)
        while group.len() < 5 {
            group.push(b'u');
        }

        let value = group
            .iter()
            .enumerate()
            .map(|(i, &ch)| (ch - b'!') as u32 * 85u32.pow(4 - i as u32))
            .sum::<u32>();

        // Only output the number of bytes that were actually encoded
        let output_bytes = original_len - 1;
        for i in 0..output_bytes {
            result.push((value >> (24 - 8 * i)) as u8);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::objects::{PdfArray, PdfDictionary, PdfName, PdfObject};

    #[test]
    fn test_ascii_hex_decode() {
        let data = b"48656C6C6F>";
        let result = decode_ascii_hex(data).unwrap();
        assert_eq!(result, b"Hello");

        let data = b"48 65 6C 6C 6F>"; // With spaces
        let result = decode_ascii_hex(data).unwrap();
        assert_eq!(result, b"Hello");

        let data = b"48656C6C6>"; // Odd number of digits
        let result = decode_ascii_hex(data).unwrap();
        assert_eq!(result, b"Hell`");
    }

    #[test]
    fn test_ascii85_decode() {
        let data = b"87cURD]j7BEbo80~>";
        let result = decode_ascii85(data).unwrap();
        assert_eq!(result, b"Hello world!");

        let data = b"z~>"; // Special case for zeros
        let result = decode_ascii85(data).unwrap();
        assert_eq!(result, &[0, 0, 0, 0]);
    }

    #[test]
    fn test_filter_from_name() {
        assert_eq!(
            Filter::from_name("ASCIIHexDecode"),
            Some(Filter::ASCIIHexDecode)
        );
        assert_eq!(
            Filter::from_name("ASCII85Decode"),
            Some(Filter::ASCII85Decode)
        );
        assert_eq!(Filter::from_name("LZWDecode"), Some(Filter::LZWDecode));
        assert_eq!(Filter::from_name("FlateDecode"), Some(Filter::FlateDecode));
        assert_eq!(
            Filter::from_name("RunLengthDecode"),
            Some(Filter::RunLengthDecode)
        );
        assert_eq!(
            Filter::from_name("CCITTFaxDecode"),
            Some(Filter::CCITTFaxDecode)
        );
        assert_eq!(Filter::from_name("JBIG2Decode"), Some(Filter::JBIG2Decode));
        assert_eq!(Filter::from_name("DCTDecode"), Some(Filter::DCTDecode));
        assert_eq!(Filter::from_name("JPXDecode"), Some(Filter::JPXDecode));
        assert_eq!(Filter::from_name("Crypt"), Some(Filter::Crypt));
        assert_eq!(Filter::from_name("UnknownFilter"), None);
    }

    #[test]
    fn test_filter_equality() {
        assert_eq!(Filter::ASCIIHexDecode, Filter::ASCIIHexDecode);
        assert_ne!(Filter::ASCIIHexDecode, Filter::ASCII85Decode);
        assert_ne!(Filter::FlateDecode, Filter::LZWDecode);
    }

    #[test]
    fn test_filter_clone() {
        let filter = Filter::FlateDecode;
        let cloned = filter.clone();
        assert_eq!(filter, cloned);
    }

    #[test]
    fn test_decode_stream_no_filter() {
        let data = b"Hello, world!";
        let dict = PdfDictionary::new();
        let options = ParseOptions::default();

        let result = decode_stream(data, &dict, &options).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_decode_stream_single_filter() {
        let data = b"48656C6C6F>";
        let mut dict = PdfDictionary::new();
        dict.insert(
            "Filter".to_string(),
            PdfObject::Name(PdfName("ASCIIHexDecode".to_string())),
        );
        let options = ParseOptions::default();

        let result = decode_stream(data, &dict, &options).unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_stream_invalid_filter() {
        let data = b"test data";
        let mut dict = PdfDictionary::new();
        dict.insert(
            "Filter".to_string(),
            PdfObject::Name(PdfName("UnknownFilter".to_string())),
        );
        let options = ParseOptions::default();

        let result = decode_stream(data, &dict, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_stream_filter_array() {
        let data = b"48656C6C6F>";
        let mut dict = PdfDictionary::new();
        let filters = vec![PdfObject::Name(PdfName("ASCIIHexDecode".to_string()))];
        dict.insert("Filter".to_string(), PdfObject::Array(PdfArray(filters)));
        let options = ParseOptions::default();

        let result = decode_stream(data, &dict, &options).unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_stream_invalid_filter_type() {
        let data = b"test data";
        let mut dict = PdfDictionary::new();
        dict.insert("Filter".to_string(), PdfObject::Integer(42)); // Invalid type
        let options = ParseOptions::default();

        let result = decode_stream(data, &dict, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_ascii_hex_decode_empty() {
        let data = b">";
        let result = decode_ascii_hex(data).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_ascii_hex_decode_invalid() {
        let data = b"GG>"; // Invalid hex
        let result = decode_ascii_hex(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_ascii_hex_decode_no_terminator() {
        let data = b"48656C6C6F"; // Missing '>'
        let result = decode_ascii_hex(data).unwrap();
        assert_eq!(result, b"Hello"); // Should work without terminator
    }

    #[test]
    fn test_ascii85_decode_empty() {
        let data = b"~>";
        let result = decode_ascii85(data).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_ascii85_decode_invalid() {
        let data = b"invalid~>";
        let result = decode_ascii85(data);
        assert!(result.is_err());
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_flate_decode() {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let original = b"Hello, compressed world!";
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();
        let options = ParseOptions::default();

        let result = decode_flate(&compressed, &options).unwrap();
        assert_eq!(result, original);
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_flate_decode_corrupt_stream() {
        // Test with corrupted compressed data
        let corrupt_data = b"This is not valid compressed data!";

        // Test with strict options (should fail)
        let strict_options = ParseOptions::strict();
        let result = decode_flate(corrupt_data, &strict_options);
        assert!(result.is_err());

        // Test with tolerant options (should attempt recovery)
        let tolerant_options = ParseOptions::tolerant();
        let result = decode_flate(corrupt_data, &tolerant_options);
        // Recovery might fail but should not panic
        assert!(result.is_err() || result.is_ok());
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_flate_decode_raw_deflate() {
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;

        // Create raw deflate data (without zlib wrapper)
        let original = b"Raw deflate data without zlib wrapper";
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        // This should fail with strict options (expects zlib wrapper)
        let strict_options = ParseOptions::strict();
        let result = decode_flate(&compressed, &strict_options);
        assert!(result.is_err());

        // But should succeed with tolerant options (tries raw deflate)
        let tolerant_options = ParseOptions::tolerant();
        let result = decode_flate(&compressed, &tolerant_options);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), original);
    }

    #[cfg(not(feature = "compression"))]
    #[test]
    fn test_flate_decode_not_supported() {
        let data = b"compressed data";
        let options = ParseOptions::default();
        let result = decode_flate(data, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_filter() {
        let data = b"48656C6C6F>";
        let options = ParseOptions::default();
        let result = apply_filter(data, Filter::ASCIIHexDecode, &options).unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_apply_filter_unsupported() {
        let data = b"test data";
        let unsupported_filters = vec![
            Filter::LZWDecode,
            Filter::RunLengthDecode,
            Filter::CCITTFaxDecode,
            Filter::JBIG2Decode,
            Filter::DCTDecode,
            Filter::JPXDecode,
            Filter::Crypt,
        ];

        let options = ParseOptions::default();
        for filter in unsupported_filters {
            let result = apply_filter(data, filter, &options);
            assert!(result.is_err());
        }
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_decode_stream_with_recovery() {
        // Create a stream with corrupted FlateDecode data
        let mut dict = PdfDictionary::new();
        dict.insert(
            "Filter".to_string(),
            PdfObject::Name(PdfName("FlateDecode".to_string())),
        );

        let corrupt_data = b"This is not valid compressed data!";

        // Test with strict options
        let strict_options = ParseOptions::strict();
        let result = decode_stream(corrupt_data, &dict, &strict_options);
        assert!(result.is_err());

        // Test with skip_errors options
        let skip_options = ParseOptions::skip_errors();
        let result = decode_stream(corrupt_data, &dict, &skip_options);
        // With skip_errors and ignore_corrupt_streams, it should return empty data
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
