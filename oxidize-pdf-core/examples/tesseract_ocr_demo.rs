//! Tesseract OCR Demo
//!
//! This example demonstrates the full capabilities of the TesseractOcrProvider
//! including installation verification, multi-language support, configuration
//! options, and performance comparison with MockOcrProvider.
//!
//! ## Prerequisites
//!
//! Before running this example, ensure Tesseract is installed on your system:
//!
//! ### macOS
//! ```bash
//! brew install tesseract
//! brew install tesseract-lang  # For additional languages
//! ```
//!
//! ### Ubuntu/Debian
//! ```bash
//! sudo apt-get install tesseract-ocr
//! sudo apt-get install tesseract-ocr-spa  # For Spanish
//! sudo apt-get install tesseract-ocr-deu  # For German
//! ```
//!
//! ### Windows
//! Download from: https://github.com/UB-Mannheim/tesseract/wiki
//!
//! ## Usage
//!
//! Compile and run with the tesseract feature:
//!
//! ```bash
//! cargo run --example tesseract_ocr_demo --features ocr-tesseract
//! ```
//!
//! Or with all OCR features:
//!
//! ```bash
//! cargo run --example tesseract_ocr_demo --features ocr-full
//! ```

use oxidize_pdf::graphics::ImageFormat;
use oxidize_pdf::operations::page_analysis::{AnalysisOptions, PageContentAnalyzer};
use oxidize_pdf::parser::PdfReader;
use oxidize_pdf::text::ocr::{
    MockOcrProvider, OcrEngine, OcrOptions, OcrProvider, OcrResult, ImagePreprocessing,
};
use oxidize_pdf::Result;

#[cfg(feature = "ocr-tesseract")]
use oxidize_pdf::text::tesseract_provider::{
    TesseractOcrProvider, TesseractConfig, PageSegmentationMode, OcrEngineMode,
};

use std::path::Path;
use std::time::Instant;

fn main() -> Result<()> {
    println!("🔍 Tesseract OCR Demo - oxidize-pdf");
    println!("=====================================\n");

    // Check if Tesseract feature is enabled
    #[cfg(not(feature = "ocr-tesseract"))]
    {
        println!("❌ This example requires the 'ocr-tesseract' feature to be enabled.");
        println!("   Run with: cargo run --example tesseract_ocr_demo --features ocr-tesseract");
        return Ok(());
    }

    #[cfg(feature = "ocr-tesseract")]
    {
        // Step 1: Check Tesseract availability
        check_tesseract_availability()?;

        // Step 2: Create sample image for testing
        let sample_image = create_sample_image_data();

        // Step 3: Basic OCR demonstration
        demonstrate_basic_ocr(&sample_image)?;

        // Step 4: Configuration options
        demonstrate_configuration_options(&sample_image)?;

        // Step 5: Multi-language support
        demonstrate_multi_language_support(&sample_image)?;

        // Step 6: Performance comparison
        demonstrate_performance_comparison(&sample_image)?;

        // Step 7: Page analysis integration
        demonstrate_page_analysis_integration()?;

        // Step 8: Error handling
        demonstrate_error_handling()?;

        println!("\n✅ Tesseract OCR Demo completed successfully!");
        println!("   For more examples, check the examples directory.");
    }

    Ok(())
}

#[cfg(feature = "ocr-tesseract")]
fn check_tesseract_availability() -> Result<()> {
    println!("🔧 Checking Tesseract Installation");
    println!("-----------------------------------");

    match TesseractOcrProvider::check_availability() {
        Ok(()) => {
            println!("✅ Tesseract is available and ready to use");
            
            // Show available languages
            match TesseractOcrProvider::available_languages() {
                Ok(languages) => {
                    println!("📋 Available languages: {}", languages.join(", "));
                }
                Err(e) => {
                    println!("⚠️  Could not retrieve language list: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Tesseract not available: {}", e);
            println!("   Please install Tesseract OCR and try again.");
            return Err(e.into());
        }
    }

    println!();
    Ok(())
}

#[cfg(feature = "ocr-tesseract")]
fn create_sample_image_data() -> Vec<u8> {
    // Create a minimal JPEG header for testing
    // In a real application, you would load actual image files
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
        0x01, 0x01, 0x00, 0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43,
        0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09,
        0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
        0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20,
        0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29,
        0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32,
        0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xD9,
    ]
}

#[cfg(feature = "ocr-tesseract")]
fn demonstrate_basic_ocr(sample_image: &[u8]) -> OcrResult<()> {
    println!("📝 Basic OCR Processing");
    println!("------------------------");

    // Create provider with default settings
    let provider = TesseractOcrProvider::new()?;
    let options = OcrOptions::default();

    println!("🔍 Processing sample image...");
    let start = Instant::now();

    // Note: This will likely fail with mock image data, but demonstrates the interface
    match provider.process_image(sample_image, &options) {
        Ok(result) => {
            println!("✅ OCR processing successful!");
            println!("   Text: {}", result.text);
            println!("   Confidence: {:.1}%", result.confidence * 100.0);
            println!("   Processing time: {}ms", result.processing_time_ms);
            println!("   Fragments: {}", result.fragments.len());
        }
        Err(e) => {
            println!("⚠️  OCR processing failed (expected with mock data): {}", e);
            println!("   This is normal when using synthetic image data.");
        }
    }

    println!("   Total time: {:?}", start.elapsed());
    println!();
    Ok(())
}

#[cfg(feature = "ocr-tesseract")]
fn demonstrate_configuration_options(sample_image: &[u8]) -> OcrResult<()> {
    println!("⚙️  Configuration Options");
    println!("-------------------------");

    // Document optimized configuration
    println!("📄 Document-optimized configuration:");
    let doc_config = TesseractConfig::for_documents();
    let doc_provider = TesseractOcrProvider::with_config(doc_config)?;
    
    println!("   PSM: {:?} ({})", doc_provider.config().psm, doc_provider.config().psm.description());
    println!("   OEM: {:?} ({})", doc_provider.config().oem, doc_provider.config().oem.description());

    // Single line configuration
    println!("\n📏 Single line configuration:");
    let line_config = TesseractConfig::for_single_line();
    let line_provider = TesseractOcrProvider::with_config(line_config)?;
    
    println!("   PSM: {:?} ({})", line_provider.config().psm, line_provider.config().psm.description());
    println!("   OEM: {:?} ({})", line_provider.config().oem, line_provider.config().oem.description());

    // Custom configuration with character restrictions
    println!("\n🔢 Numbers-only configuration:");
    let custom_config = TesseractConfig::default()
        .with_char_whitelist("0123456789.,+-")
        .with_debug();
    
    let custom_provider = TesseractOcrProvider::with_config(custom_config)?;
    println!("   Character whitelist: {:?}", custom_provider.config().char_whitelist);
    println!("   Debug mode: {}", custom_provider.config().debug);

    println!();
    Ok(())
}

#[cfg(feature = "ocr-tesseract")]
fn demonstrate_multi_language_support(_sample_image: &[u8]) -> OcrResult<()> {
    println!("🌍 Multi-language Support");
    println!("--------------------------");

    // English provider
    let eng_provider = TesseractOcrProvider::with_language("eng")?;
    println!("✅ English provider created");

    // Try Spanish (if available)
    match TesseractOcrProvider::with_language("spa") {
        Ok(_) => println!("✅ Spanish provider created"),
        Err(e) => println!("⚠️  Spanish not available: {}", e),
    }

    // Try German (if available)
    match TesseractOcrProvider::with_language("deu") {
        Ok(_) => println!("✅ German provider created"),
        Err(e) => println!("⚠️  German not available: {}", e),
    }

    // Multi-language provider
    match TesseractOcrProvider::with_language("eng+spa") {
        Ok(_) => println!("✅ Multi-language (English + Spanish) provider created"),
        Err(e) => println!("⚠️  Multi-language not available: {}", e),
    }

    println!();
    Ok(())
}

#[cfg(feature = "ocr-tesseract")]
fn demonstrate_performance_comparison(sample_image: &[u8]) -> OcrResult<()> {
    println!("⚡ Performance Comparison");
    println!("-------------------------");

    let options = OcrOptions::default();
    let iterations = 3;

    // Test MockOcrProvider
    println!("🧪 MockOcrProvider performance:");
    let mock_provider = MockOcrProvider::new();
    let mut mock_total_time = 0u64;

    for i in 1..=iterations {
        let start = Instant::now();
        match mock_provider.process_image(sample_image, &options) {
            Ok(result) => {
                let elapsed = start.elapsed();
                mock_total_time += elapsed.as_millis() as u64;
                println!("   Run {}: {}ms (reported: {}ms)", i, elapsed.as_millis(), result.processing_time_ms);
            }
            Err(e) => println!("   Run {}: Error - {}", i, e),
        }
    }

    println!("   Average: {}ms", mock_total_time / iterations as u64);

    // Test TesseractOcrProvider
    println!("\n🔍 TesseractOcrProvider performance:");
    let tesseract_provider = TesseractOcrProvider::new()?;
    let mut tesseract_total_time = 0u64;

    for i in 1..=iterations {
        let start = Instant::now();
        match tesseract_provider.process_image(sample_image, &options) {
            Ok(result) => {
                let elapsed = start.elapsed();
                tesseract_total_time += elapsed.as_millis() as u64;
                println!("   Run {}: {}ms (reported: {}ms)", i, elapsed.as_millis(), result.processing_time_ms);
            }
            Err(e) => {
                let elapsed = start.elapsed();
                tesseract_total_time += elapsed.as_millis() as u64;
                println!("   Run {}: {}ms (Error: {})", i, elapsed.as_millis(), e);
            }
        }
    }

    println!("   Average: {}ms", tesseract_total_time / iterations as u64);

    println!();
    Ok(())
}

#[cfg(feature = "ocr-tesseract")]
fn demonstrate_page_analysis_integration() -> OcrResult<()> {
    println!("📊 Page Analysis Integration");
    println!("-----------------------------");

    // Try to find a PDF file for testing
    let test_files = ["test.pdf", "sample.pdf", "document.pdf"];
    let mut found_pdf = None;

    for file in &test_files {
        if Path::new(file).exists() {
            found_pdf = Some(file);
            break;
        }
    }

    match found_pdf {
        Some(pdf_path) => {
            println!("📄 Processing PDF: {}", pdf_path);
            
            match process_pdf_with_ocr(pdf_path) {
                Ok(()) => println!("✅ PDF processing completed successfully"),
                Err(e) => println!("⚠️  PDF processing failed: {}", e),
            }
        }
        None => {
            println!("⚠️  No test PDF files found. Skipping page analysis integration.");
            println!("   To test this feature, place a PDF file named 'test.pdf' in the current directory.");
        }
    }

    println!();
    Ok(())
}

#[cfg(feature = "ocr-tesseract")]
fn process_pdf_with_ocr(pdf_path: &str) -> Result<()> {
    // Open PDF document
    let document = PdfReader::open_document(pdf_path)?;
    
    // Create page analyzer
    let mut analyzer = PageContentAnalyzer::new(document);
    
    // Configure OCR options
    let ocr_options = OcrOptions {
        language: "eng".to_string(),
        min_confidence: 0.7,
        preserve_layout: true,
        preprocessing: ImagePreprocessing {
            denoise: true,
            deskew: true,
            enhance_contrast: true,
            sharpen: false,
            scale_factor: 1.0,
        },
        ..Default::default()
    };
    
    // Update analysis options with OCR
    let analysis_options = AnalysisOptions {
        ocr_options: Some(ocr_options),
        ..Default::default()
    };
    
    analyzer.set_options(analysis_options);
    
    // Create OCR provider
    let ocr_provider = TesseractOcrProvider::new()?;
    
    // Find scanned pages
    let scanned_pages = analyzer.find_scanned_pages()?;
    
    if scanned_pages.is_empty() {
        println!("   No scanned pages found in document");
        return Ok(());
    }
    
    println!("   Found {} scanned pages: {:?}", scanned_pages.len(), scanned_pages);
    
    // Process each scanned page
    for page_num in scanned_pages.iter().take(3) { // Process first 3 pages max
        match analyzer.extract_text_from_scanned_page(*page_num, &ocr_provider) {
            Ok(ocr_result) => {
                println!("   Page {}: {} characters extracted ({:.1}% confidence)", 
                         page_num, ocr_result.text.len(), ocr_result.confidence * 100.0);
            }
            Err(e) => {
                println!("   Page {}: OCR failed - {}", page_num, e);
            }
        }
    }
    
    Ok(())
}

#[cfg(feature = "ocr-tesseract")]
fn demonstrate_error_handling() -> OcrResult<()> {
    println!("🚨 Error Handling");
    println!("------------------");

    // Test with invalid image data
    println!("❌ Testing with invalid image data:");
    let invalid_data = vec![0x00, 0x01, 0x02, 0x03];
    let provider = TesseractOcrProvider::new()?;
    let options = OcrOptions::default();

    match provider.process_image(&invalid_data, &options) {
        Ok(_) => println!("   Unexpected: processing succeeded"),
        Err(e) => println!("   Expected error: {}", e),
    }

    // Test with low confidence threshold
    println!("\n🎯 Testing with high confidence threshold:");
    let high_confidence_options = OcrOptions {
        min_confidence: 0.99, // Very high threshold
        ..Default::default()
    };

    let sample_image = create_sample_image_data();
    match provider.process_image(&sample_image, &high_confidence_options) {
        Ok(result) => println!("   Confidence: {:.1}%", result.confidence * 100.0),
        Err(e) => println!("   Expected error: {}", e),
    }

    // Test with unsupported language
    println!("\n🌐 Testing with unsupported language:");
    match TesseractOcrProvider::with_language("nonexistent") {
        Ok(_) => println!("   Unexpected: language supported"),
        Err(e) => println!("   Expected error: {}", e),
    }

    println!();
    Ok(())
}

#[cfg(not(feature = "ocr-tesseract"))]
fn main() -> Result<()> {
    println!("❌ This example requires the 'ocr-tesseract' feature to be enabled.");
    println!("   Run with: cargo run --example tesseract_ocr_demo --features ocr-tesseract");
    Ok(())
}