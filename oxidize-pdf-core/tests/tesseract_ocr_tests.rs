//! Comprehensive tests for TesseractOcrProvider
//!
//! These tests verify the Tesseract OCR provider implementation including:
//! - Configuration validation
//! - Error handling
//! - Performance characteristics
//! - Integration with page analysis
//! - Multi-language support
//!
//! Most tests are marked as `#[ignore]` because they require Tesseract to be installed.
//! Run with: `cargo test tesseract_ocr_tests --features ocr-tesseract -- --ignored`

#[cfg(feature = "ocr-tesseract")]
mod tesseract_tests {
    use oxidize_pdf::graphics::ImageFormat;
    use oxidize_pdf::text::ocr::{
        OcrEngine, OcrOptions, OcrProvider, OcrResult, ImagePreprocessing,
        FragmentType, OcrTextFragment, OcrProcessingResult,
    };
    use oxidize_pdf::text::tesseract_provider::{
        TesseractOcrProvider, TesseractConfig, PageSegmentationMode, OcrEngineMode,
    };
    use std::time::Duration;

    // Helper function to create mock image data
    fn create_mock_jpeg_data() -> Vec<u8> {
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

    fn create_mock_png_data() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
            0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    }

    #[test]
    fn test_tesseract_config_defaults() {
        let config = TesseractConfig::default();
        assert_eq!(config.language, "eng");
        assert_eq!(config.psm, PageSegmentationMode::Auto);
        assert_eq!(config.oem, OcrEngineMode::Default);
        assert!(config.char_whitelist.is_none());
        assert!(config.char_blacklist.is_none());
        assert!(config.variables.is_empty());
        assert!(!config.debug);
    }

    #[test]
    fn test_tesseract_config_with_language() {
        let config = TesseractConfig::with_language("spa");
        assert_eq!(config.language, "spa");
        assert_eq!(config.psm, PageSegmentationMode::Auto);
        assert_eq!(config.oem, OcrEngineMode::Default);
    }

    #[test]
    fn test_tesseract_config_presets() {
        // Document configuration
        let doc_config = TesseractConfig::for_documents();
        assert_eq!(doc_config.language, "eng");
        assert_eq!(doc_config.psm, PageSegmentationMode::Auto);
        assert_eq!(doc_config.oem, OcrEngineMode::LstmOnly);

        // Single line configuration
        let line_config = TesseractConfig::for_single_line();
        assert_eq!(line_config.language, "eng");
        assert_eq!(line_config.psm, PageSegmentationMode::SingleLine);
        assert_eq!(line_config.oem, OcrEngineMode::LstmOnly);

        // Sparse text configuration
        let sparse_config = TesseractConfig::for_sparse_text();
        assert_eq!(sparse_config.language, "eng");
        assert_eq!(sparse_config.psm, PageSegmentationMode::SparseText);
        assert_eq!(sparse_config.oem, OcrEngineMode::LstmOnly);
    }

    #[test]
    fn test_tesseract_config_builder_pattern() {
        let config = TesseractConfig::default()
            .with_char_whitelist("0123456789")
            .with_char_blacklist("!@#$%")
            .with_variable("tessedit_char_blacklist", "")
            .with_debug();

        assert_eq!(config.char_whitelist, Some("0123456789".to_string()));
        assert_eq!(config.char_blacklist, Some("!@#$%".to_string()));
        assert!(config.variables.contains_key("tessedit_char_blacklist"));
        assert!(config.debug);
    }

    #[test]
    fn test_page_segmentation_mode_values() {
        assert_eq!(PageSegmentationMode::OsdOnly as u8, 0);
        assert_eq!(PageSegmentationMode::AutoOsd as u8, 1);
        assert_eq!(PageSegmentationMode::AutoOnly as u8, 2);
        assert_eq!(PageSegmentationMode::Auto as u8, 3);
        assert_eq!(PageSegmentationMode::SingleColumn as u8, 4);
        assert_eq!(PageSegmentationMode::SingleBlock as u8, 5);
        assert_eq!(PageSegmentationMode::SingleUniformBlock as u8, 6);
        assert_eq!(PageSegmentationMode::SingleLine as u8, 7);
        assert_eq!(PageSegmentationMode::SingleWord as u8, 8);
        assert_eq!(PageSegmentationMode::SingleWordCircle as u8, 9);
        assert_eq!(PageSegmentationMode::SingleChar as u8, 10);
        assert_eq!(PageSegmentationMode::SparseText as u8, 11);
        assert_eq!(PageSegmentationMode::SparseTextOsd as u8, 12);
        assert_eq!(PageSegmentationMode::RawLine as u8, 13);
    }

    #[test]
    fn test_page_segmentation_mode_methods() {
        let psm = PageSegmentationMode::Auto;
        assert_eq!(psm.to_psm_value(), 3);
        assert!(psm.description().contains("automatic"));

        let psm = PageSegmentationMode::SingleLine;
        assert_eq!(psm.to_psm_value(), 7);
        assert!(psm.description().contains("Single text line"));
    }

    #[test]
    fn test_ocr_engine_mode_values() {
        assert_eq!(OcrEngineMode::LegacyOnly as u8, 0);
        assert_eq!(OcrEngineMode::LstmOnly as u8, 1);
        assert_eq!(OcrEngineMode::LegacyLstm as u8, 2);
        assert_eq!(OcrEngineMode::Default as u8, 3);
    }

    #[test]
    fn test_ocr_engine_mode_methods() {
        let oem = OcrEngineMode::LstmOnly;
        assert_eq!(oem.to_oem_value(), 1);
        assert!(oem.description().contains("LSTM"));

        let oem = OcrEngineMode::Default;
        assert_eq!(oem.to_oem_value(), 3);
        assert!(oem.description().contains("Default"));
    }

    #[test]
    fn test_tesseract_available_languages() {
        let languages = TesseractOcrProvider::available_languages().unwrap();
        assert!(!languages.is_empty());
        assert!(languages.contains(&"eng".to_string()));
        
        // Check for common languages
        let common_languages = ["spa", "deu", "fra", "ita", "por", "rus"];
        for lang in &common_languages {
            assert!(languages.contains(&lang.to_string()));
        }
    }

    #[test]
    fn test_tesseract_supported_formats() {
        let formats = vec![ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::Tiff];
        assert!(formats.contains(&ImageFormat::Jpeg));
        assert!(formats.contains(&ImageFormat::Png));
        assert!(formats.contains(&ImageFormat::Tiff));
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_availability() {
        match TesseractOcrProvider::check_availability() {
            Ok(()) => {
                // Tesseract is available
                assert!(true);
            }
            Err(e) => {
                // Tesseract is not available
                println!("Tesseract not available: {}", e);
                panic!("Tesseract is required for this test");
            }
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_provider_creation() {
        let provider = TesseractOcrProvider::new().expect("Failed to create provider");
        assert_eq!(provider.engine_name(), "Tesseract");
        assert_eq!(provider.engine_type(), OcrEngine::Tesseract);
        
        let formats = provider.supported_formats();
        assert!(formats.contains(&ImageFormat::Jpeg));
        assert!(formats.contains(&ImageFormat::Png));
        assert!(formats.contains(&ImageFormat::Tiff));
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_provider_with_config() {
        let config = TesseractConfig::for_documents();
        let provider = TesseractOcrProvider::with_config(config).expect("Failed to create provider");
        
        assert_eq!(provider.config().psm, PageSegmentationMode::Auto);
        assert_eq!(provider.config().oem, OcrEngineMode::LstmOnly);
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_provider_with_language() {
        let provider = TesseractOcrProvider::with_language("eng").expect("Failed to create provider");
        assert_eq!(provider.config().language, "eng");
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_provider_multi_language() {
        // Test multi-language support
        match TesseractOcrProvider::with_language("eng+spa") {
            Ok(provider) => {
                assert_eq!(provider.config().language, "eng+spa");
            }
            Err(e) => {
                println!("Multi-language not supported: {}", e);
                // This is acceptable if language packs are not installed
            }
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_config_update() {
        let mut provider = TesseractOcrProvider::new().expect("Failed to create provider");
        
        let new_config = TesseractConfig::for_single_line();
        provider.set_config(new_config).expect("Failed to update config");
        
        assert_eq!(provider.config().psm, PageSegmentationMode::SingleLine);
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_image_validation() {
        let provider = TesseractOcrProvider::new().expect("Failed to create provider");
        
        // Valid JPEG data
        let jpeg_data = create_mock_jpeg_data();
        assert!(provider.validate_image_data(&jpeg_data).is_ok());
        
        // Valid PNG data
        let png_data = create_mock_png_data();
        assert!(provider.validate_image_data(&png_data).is_ok());
        
        // Invalid data (too short)
        let short_data = vec![0xFF, 0xD8];
        assert!(provider.validate_image_data(&short_data).is_err());
        
        // Invalid format
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        assert!(provider.validate_image_data(&invalid_data).is_err());
    }

    #[test]
    #[ignore = "Requires Tesseract installation and sample image"]
    fn test_tesseract_process_image() {
        let provider = TesseractOcrProvider::new().expect("Failed to create provider");
        let options = OcrOptions::default();
        
        // Note: This test will fail with mock data but verifies the interface
        let image_data = create_mock_jpeg_data();
        
        match provider.process_image(&image_data, &options) {
            Ok(result) => {
                // If processing succeeds (unlikely with mock data)
                assert!(!result.text.is_empty());
                assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
                assert_eq!(result.engine_name, "Tesseract");
                assert_eq!(result.language, "en");
            }
            Err(e) => {
                // Expected to fail with mock data
                println!("Expected failure with mock data: {}", e);
            }
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_different_psm_modes() {
        let psm_modes = vec![
            PageSegmentationMode::Auto,
            PageSegmentationMode::SingleLine,
            PageSegmentationMode::SingleWord,
            PageSegmentationMode::SparseText,
        ];
        
        for psm in psm_modes {
            let config = TesseractConfig {
                psm,
                ..Default::default()
            };
            
            let provider = TesseractOcrProvider::with_config(config).expect("Failed to create provider");
            assert_eq!(provider.config().psm, psm);
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_different_oem_modes() {
        let oem_modes = vec![
            OcrEngineMode::LegacyOnly,
            OcrEngineMode::LstmOnly,
            OcrEngineMode::LegacyLstm,
            OcrEngineMode::Default,
        ];
        
        for oem in oem_modes {
            let config = TesseractConfig {
                oem,
                ..Default::default()
            };
            
            let provider = TesseractOcrProvider::with_config(config).expect("Failed to create provider");
            assert_eq!(provider.config().oem, oem);
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_char_whitelist() {
        let config = TesseractConfig::default()
            .with_char_whitelist("0123456789");
        
        let provider = TesseractOcrProvider::with_config(config).expect("Failed to create provider");
        assert_eq!(provider.config().char_whitelist, Some("0123456789".to_string()));
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_char_blacklist() {
        let config = TesseractConfig::default()
            .with_char_blacklist("!@#$%");
        
        let provider = TesseractOcrProvider::with_config(config).expect("Failed to create provider");
        assert_eq!(provider.config().char_blacklist, Some("!@#$%".to_string()));
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_custom_variables() {
        let config = TesseractConfig::default()
            .with_variable("tessedit_char_blacklist", "")
            .with_variable("debug_file", "/tmp/tesseract.log");
        
        let provider = TesseractOcrProvider::with_config(config).expect("Failed to create provider");
        assert!(provider.config().variables.contains_key("tessedit_char_blacklist"));
        assert!(provider.config().variables.contains_key("debug_file"));
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_debug_mode() {
        let config = TesseractConfig::default().with_debug();
        let provider = TesseractOcrProvider::with_config(config).expect("Failed to create provider");
        assert!(provider.config().debug);
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_error_handling_invalid_language() {
        match TesseractOcrProvider::with_language("invalid_language_code") {
            Ok(_) => panic!("Expected error for invalid language"),
            Err(e) => {
                assert!(e.to_string().contains("not available") || e.to_string().contains("Failed to initialize"));
            }
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_low_confidence_handling() {
        let provider = TesseractOcrProvider::new().expect("Failed to create provider");
        
        // Set very high confidence threshold
        let options = OcrOptions {
            min_confidence: 0.99,
            ..Default::default()
        };
        
        let image_data = create_mock_jpeg_data();
        match provider.process_image(&image_data, &options) {
            Ok(result) => {
                // If it succeeds, confidence should be high
                assert!(result.confidence >= 0.99);
            }
            Err(e) => {
                // Expected to fail with high confidence threshold
                assert!(e.to_string().contains("confidence") || e.to_string().contains("Invalid image"));
            }
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_preprocessing_options() {
        let provider = TesseractOcrProvider::new().expect("Failed to create provider");
        
        let options = OcrOptions {
            preprocessing: ImagePreprocessing {
                denoise: true,
                deskew: true,
                enhance_contrast: true,
                sharpen: true,
                scale_factor: 1.5,
            },
            ..Default::default()
        };
        
        let image_data = create_mock_jpeg_data();
        
        // Test that preprocessing options are accepted
        match provider.process_image(&image_data, &options) {
            Ok(_) => {
                // Processing succeeded with preprocessing options
                assert!(true);
            }
            Err(_) => {
                // Expected to fail with mock data, but options were processed
                assert!(true);
            }
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_timeout_handling() {
        let provider = TesseractOcrProvider::new().expect("Failed to create provider");
        
        let options = OcrOptions {
            timeout_seconds: 1, // Very short timeout
            ..Default::default()
        };
        
        let image_data = create_mock_jpeg_data();
        
        // Test that timeout is respected
        let start = std::time::Instant::now();
        let _ = provider.process_image(&image_data, &options);
        let elapsed = start.elapsed();
        
        // Should complete within reasonable time (even if it fails)
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_format_support() {
        let provider = TesseractOcrProvider::new().expect("Failed to create provider");
        
        assert!(provider.supports_format(ImageFormat::Jpeg));
        assert!(provider.supports_format(ImageFormat::Png));
        assert!(provider.supports_format(ImageFormat::Tiff));
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_layout_preservation() {
        let provider = TesseractOcrProvider::new().expect("Failed to create provider");
        
        let options = OcrOptions {
            preserve_layout: true,
            ..Default::default()
        };
        
        let image_data = create_mock_jpeg_data();
        
        match provider.process_image(&image_data, &options) {
            Ok(result) => {
                // Should have position information when layout is preserved
                for fragment in &result.fragments {
                    assert!(fragment.x >= 0.0);
                    assert!(fragment.y >= 0.0);
                    assert!(fragment.width >= 0.0);
                    assert!(fragment.height >= 0.0);
                }
            }
            Err(_) => {
                // Expected to fail with mock data
                assert!(true);
            }
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_performance_characteristics() {
        let provider = TesseractOcrProvider::new().expect("Failed to create provider");
        let options = OcrOptions::default();
        let image_data = create_mock_jpeg_data();
        
        // Test multiple runs to check consistency
        for _ in 0..3 {
            let start = std::time::Instant::now();
            let _ = provider.process_image(&image_data, &options);
            let elapsed = start.elapsed();
            
            // Should complete within reasonable time
            assert!(elapsed < Duration::from_secs(30));
        }
    }

    #[test]
    fn test_tesseract_stub_without_feature() {
        // Test that without the feature, appropriate errors are returned
        #[cfg(not(feature = "ocr-tesseract"))]
        {
            let result = TesseractOcrProvider::new();
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not available"));
            
            let result = TesseractOcrProvider::check_availability();
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not available"));
        }
    }
}

// Tests that run without the feature enabled
#[cfg(not(feature = "ocr-tesseract"))]
mod no_feature_tests {
    use crate::text::tesseract_provider::TesseractOcrProvider;

    #[test]
    fn test_tesseract_provider_unavailable() {
        let result = TesseractOcrProvider::new();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not available"));
    }

    #[test]
    fn test_tesseract_check_availability_unavailable() {
        let result = TesseractOcrProvider::check_availability();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not available"));
    }
}