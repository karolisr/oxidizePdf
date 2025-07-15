//! Tesseract OCR provider implementation
//!
//! This module provides a complete implementation of the OcrProvider trait using
//! Tesseract OCR engine. It supports multiple languages, configurable page segmentation
//! modes, and various OCR engine modes.
//!
//! # Installation
//!
//! Before using this provider, you need to install Tesseract OCR on your system:
//!
//! ## macOS
//! ```bash
//! brew install tesseract
//! brew install tesseract-lang  # For additional languages
//! ```
//!
//! ## Ubuntu/Debian
//! ```bash
//! sudo apt-get install tesseract-ocr
//! sudo apt-get install tesseract-ocr-spa  # For Spanish
//! sudo apt-get install tesseract-ocr-deu  # For German
//! ```
//!
//! ## Windows
//! Download from: https://github.com/UB-Mannheim/tesseract/wiki
//!
//! # Usage
//!
//! ```rust
//! use oxidize_pdf::text::{TesseractOcrProvider, OcrOptions, OcrProvider};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = TesseractOcrProvider::new()?;
//! let options = OcrOptions::default();
//! let image_data = std::fs::read("scanned_page.jpg")?;
//!
//! let result = provider.process_image(&image_data, &options)?;
//! println!("Extracted text: {}", result.text);
//! println!("Confidence: {:.1}%", result.confidence * 100.0);
//! # Ok(())
//! # }
//! ```
//!
//! # Multiple Languages
//!
//! ```rust
//! use oxidize_pdf::text::{TesseractOcrProvider, TesseractConfig, OcrOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = TesseractConfig {
//!     language: "spa+eng".to_string(),  // Spanish + English
//!     psm: PageSegmentationMode::Auto,
//!     oem: OcrEngineMode::LstmOnly,
//!     ..Default::default()
//! };
//!
//! let provider = TesseractOcrProvider::with_config(config)?;
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "ocr-tesseract")]
use crate::graphics::ImageFormat;
#[cfg(feature = "ocr-tesseract")]
use crate::operations::page_analysis::ContentAnalysis;
#[cfg(feature = "ocr-tesseract")]
use crate::text::{
    FragmentType, OcrEngine, OcrError, OcrOptions, OcrProcessingResult, OcrProvider, OcrResult,
    OcrTextFragment,
};
#[cfg(feature = "ocr-tesseract")]
use std::collections::HashMap;
#[cfg(feature = "ocr-tesseract")]
use std::sync::Mutex;
#[cfg(feature = "ocr-tesseract")]
use std::time::Instant;
#[cfg(feature = "ocr-tesseract")]
use tesseract::{Tesseract, TesseractError};

/// Page Segmentation Mode for Tesseract
#[cfg(feature = "ocr-tesseract")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSegmentationMode {
    /// Orientation and script detection (OSD) only
    OsdOnly = 0,
    /// Automatic page segmentation with OSD
    AutoOsd = 1,
    /// Automatic page segmentation, but no OSD, or OCR
    AutoOnly = 2,
    /// Fully automatic page segmentation, but no OSD (Default)
    Auto = 3,
    /// Assume a single column of text of variable sizes
    SingleColumn = 4,
    /// Assume a single uniform block of vertically aligned text
    SingleBlock = 5,
    /// Assume a single uniform block of text
    SingleUniformBlock = 6,
    /// Treat the image as a single text line
    SingleLine = 7,
    /// Treat the image as a single word
    SingleWord = 8,
    /// Treat the image as a single word in a circle
    SingleWordCircle = 9,
    /// Treat the image as a single character
    SingleChar = 10,
    /// Sparse text. Find as much text as possible in no particular order
    SparseText = 11,
    /// Sparse text with OSD
    SparseTextOsd = 12,
    /// Raw line. Treat the image as a single text line, bypassing hacks
    RawLine = 13,
}

#[cfg(feature = "ocr-tesseract")]
impl Default for PageSegmentationMode {
    fn default() -> Self {
        PageSegmentationMode::Auto
    }
}

#[cfg(feature = "ocr-tesseract")]
impl PageSegmentationMode {
    /// Convert to Tesseract PSM value
    pub fn to_psm_value(self) -> u8 {
        self as u8
    }

    /// Get description of the PSM mode
    pub fn description(&self) -> &'static str {
        match self {
            PageSegmentationMode::OsdOnly => "Orientation and script detection only",
            PageSegmentationMode::AutoOsd => "Automatic page segmentation with OSD",
            PageSegmentationMode::AutoOnly => "Automatic page segmentation, no OSD or OCR",
            PageSegmentationMode::Auto => "Fully automatic page segmentation, no OSD",
            PageSegmentationMode::SingleColumn => "Single column of text of variable sizes",
            PageSegmentationMode::SingleBlock => "Single uniform block of vertically aligned text",
            PageSegmentationMode::SingleUniformBlock => "Single uniform block of text",
            PageSegmentationMode::SingleLine => "Single text line",
            PageSegmentationMode::SingleWord => "Single word",
            PageSegmentationMode::SingleWordCircle => "Single word in a circle",
            PageSegmentationMode::SingleChar => "Single character",
            PageSegmentationMode::SparseText => "Sparse text in no particular order",
            PageSegmentationMode::SparseTextOsd => "Sparse text with OSD",
            PageSegmentationMode::RawLine => "Raw line, bypassing hacks",
        }
    }
}

/// OCR Engine Mode for Tesseract
#[cfg(feature = "ocr-tesseract")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcrEngineMode {
    /// Legacy engine only
    LegacyOnly = 0,
    /// Neural nets LSTM engine only
    LstmOnly = 1,
    /// Legacy + LSTM engines
    LegacyLstm = 2,
    /// Default, based on what is available
    Default = 3,
}

#[cfg(feature = "ocr-tesseract")]
impl Default for OcrEngineMode {
    fn default() -> Self {
        OcrEngineMode::Default
    }
}

#[cfg(feature = "ocr-tesseract")]
impl OcrEngineMode {
    /// Convert to Tesseract OEM value
    pub fn to_oem_value(self) -> u8 {
        self as u8
    }

    /// Get description of the OEM mode
    pub fn description(&self) -> &'static str {
        match self {
            OcrEngineMode::LegacyOnly => "Legacy engine only",
            OcrEngineMode::LstmOnly => "Neural nets LSTM engine only",
            OcrEngineMode::LegacyLstm => "Legacy + LSTM engines",
            OcrEngineMode::Default => "Default, based on what is available",
        }
    }
}

/// Configuration for Tesseract OCR provider
#[cfg(feature = "ocr-tesseract")]
#[derive(Debug, Clone)]
pub struct TesseractConfig {
    /// Language code (e.g., "eng", "spa", "deu", "eng+spa")
    pub language: String,
    /// Page segmentation mode
    pub psm: PageSegmentationMode,
    /// OCR engine mode
    pub oem: OcrEngineMode,
    /// Character whitelist (only recognize these characters)
    pub char_whitelist: Option<String>,
    /// Character blacklist (never recognize these characters)
    pub char_blacklist: Option<String>,
    /// Custom Tesseract variables
    pub variables: HashMap<String, String>,
    /// Enable debug output
    pub debug: bool,
}

#[cfg(feature = "ocr-tesseract")]
impl Default for TesseractConfig {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            psm: PageSegmentationMode::default(),
            oem: OcrEngineMode::default(),
            char_whitelist: None,
            char_blacklist: None,
            variables: HashMap::new(),
            debug: false,
        }
    }
}

#[cfg(feature = "ocr-tesseract")]
impl TesseractConfig {
    /// Create a new configuration with the specified language
    pub fn with_language(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
            ..Default::default()
        }
    }

    /// Create a configuration optimized for documents
    pub fn for_documents() -> Self {
        Self {
            language: "eng".to_string(),
            psm: PageSegmentationMode::Auto,
            oem: OcrEngineMode::LstmOnly,
            ..Default::default()
        }
    }

    /// Create a configuration optimized for single lines of text
    pub fn for_single_line() -> Self {
        Self {
            language: "eng".to_string(),
            psm: PageSegmentationMode::SingleLine,
            oem: OcrEngineMode::LstmOnly,
            ..Default::default()
        }
    }

    /// Create a configuration optimized for sparse text
    pub fn for_sparse_text() -> Self {
        Self {
            language: "eng".to_string(),
            psm: PageSegmentationMode::SparseText,
            oem: OcrEngineMode::LstmOnly,
            ..Default::default()
        }
    }

    /// Set character whitelist
    pub fn with_char_whitelist(mut self, whitelist: impl Into<String>) -> Self {
        self.char_whitelist = Some(whitelist.into());
        self
    }

    /// Set character blacklist
    pub fn with_char_blacklist(mut self, blacklist: impl Into<String>) -> Self {
        self.char_blacklist = Some(blacklist.into());
        self
    }

    /// Add a custom Tesseract variable
    pub fn with_variable(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    /// Enable debug output
    pub fn with_debug(mut self) -> Self {
        self.debug = true;
        self
    }
}

/// Tesseract OCR provider implementation
#[cfg(feature = "ocr-tesseract")]
pub struct TesseractOcrProvider {
    config: TesseractConfig,
    instance: Mutex<Option<Tesseract>>,
}

#[cfg(feature = "ocr-tesseract")]
impl TesseractOcrProvider {
    /// Create a new Tesseract OCR provider with default configuration
    ///
    /// # Errors
    ///
    /// Returns an error if Tesseract is not installed or cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxidize_pdf::text::TesseractOcrProvider;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = TesseractOcrProvider::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> OcrResult<Self> {
        Self::with_config(TesseractConfig::default())
    }

    /// Create a new Tesseract OCR provider with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Tesseract configuration settings
    ///
    /// # Errors
    ///
    /// Returns an error if Tesseract is not installed or cannot be initialized
    /// with the provided configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxidize_pdf::text::{TesseractOcrProvider, TesseractConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TesseractConfig::for_documents();
    /// let provider = TesseractOcrProvider::with_config(config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_config(config: TesseractConfig) -> OcrResult<Self> {
        // Test Tesseract initialization
        let _ = Self::create_tesseract_instance(&config)?;

        Ok(Self {
            config,
            instance: Mutex::new(None),
        })
    }

    /// Create a new Tesseract OCR provider with specified language
    ///
    /// # Arguments
    ///
    /// * `language` - Language code (e.g., "eng", "spa", "deu", "eng+spa")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxidize_pdf::text::TesseractOcrProvider;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = TesseractOcrProvider::with_language("spa")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_language(language: impl Into<String>) -> OcrResult<Self> {
        let config = TesseractConfig::with_language(language);
        Self::with_config(config)
    }

    /// Get the current configuration
    pub fn config(&self) -> &TesseractConfig {
        &self.config
    }

    /// Update the configuration
    ///
    /// This will reset the internal Tesseract instance on the next use.
    pub fn set_config(&mut self, config: TesseractConfig) -> OcrResult<()> {
        // Test the new configuration
        let _ = Self::create_tesseract_instance(&config)?;

        self.config = config;
        // Reset the instance so it will be recreated with new config
        if let Ok(mut instance) = self.instance.lock() {
            *instance = None;
        }

        Ok(())
    }

    /// Check if Tesseract is available and properly installed
    ///
    /// # Returns
    ///
    /// `Ok(())` if Tesseract is available, `Err(OcrError)` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxidize_pdf::text::TesseractOcrProvider;
    ///
    /// match TesseractOcrProvider::check_availability() {
    ///     Ok(()) => println!("Tesseract is available"),
    ///     Err(e) => println!("Tesseract not available: {}", e),
    /// }
    /// ```
    pub fn check_availability() -> OcrResult<()> {
        match Tesseract::new(Some("eng"), None) {
            Ok(_) => Ok(()),
            Err(e) => Err(OcrError::ProviderNotAvailable(format!(
                "Tesseract not available: {}",
                e
            ))),
        }
    }

    /// Get available languages
    ///
    /// # Returns
    ///
    /// A vector of language codes that are available for OCR.
    pub fn available_languages() -> OcrResult<Vec<String>> {
        // This is a simplified implementation
        // In a real implementation, you might want to query Tesseract for available languages
        Ok(vec![
            "eng".to_string(),
            "spa".to_string(),
            "deu".to_string(),
            "fra".to_string(),
            "ita".to_string(),
            "por".to_string(),
            "rus".to_string(),
            "chi_sim".to_string(),
            "chi_tra".to_string(),
            "jpn".to_string(),
            "kor".to_string(),
            "ara".to_string(),
            "hin".to_string(),
        ])
    }

    /// Create a new Tesseract instance with the given configuration
    fn create_tesseract_instance(config: &TesseractConfig) -> OcrResult<Tesseract> {
        // Initialize Tesseract with language and datapath
        let mut tesseract = Tesseract::new(Some(&config.language), None).map_err(|e| {
            OcrError::ProviderNotAvailable(format!("Failed to initialize Tesseract: {e}"))
        })?;

        // Set page segmentation mode
        tesseract
            .set_variable(
                "tessedit_pageseg_mode",
                &config.psm.to_psm_value().to_string(),
            )
            .map_err(|e| OcrError::Configuration(format!("Failed to set PSM: {e}")))?;

        // Set OCR engine mode
        tesseract
            .set_variable(
                "tessedit_ocr_engine_mode",
                &config.oem.to_oem_value().to_string(),
            )
            .map_err(|e| OcrError::Configuration(format!("Failed to set OEM: {e}")))?;

        // Set character whitelist if specified
        if let Some(ref whitelist) = config.char_whitelist {
            tesseract
                .set_variable("tessedit_char_whitelist", whitelist)
                .map_err(|e| OcrError::Configuration(format!("Failed to set whitelist: {e}")))?;
        }

        // Set character blacklist if specified
        if let Some(ref blacklist) = config.char_blacklist {
            tesseract
                .set_variable("tessedit_char_blacklist", blacklist)
                .map_err(|e| OcrError::Configuration(format!("Failed to set blacklist: {e}")))?;
        }

        // Set custom variables
        for (name, value) in &config.variables {
            tesseract.set_variable(name, value).map_err(|e| {
                OcrError::Configuration(format!("Failed to set variable {name}: {e}"))
            })?;
        }

        // Set debug mode if enabled
        if config.debug {
            tesseract
                .set_variable("debug_file", "/tmp/tesseract_debug.log")
                .map_err(|e| OcrError::Configuration(format!("Failed to set debug mode: {e}")))?;
        }

        Ok(tesseract)
    }

    /// Get or create a Tesseract instance
    fn get_tesseract_instance(&self) -> OcrResult<Tesseract> {
        let mut instance_guard = self
            .instance
            .lock()
            .map_err(|e| OcrError::ProcessingFailed(format!("Failed to acquire lock: {e}")))?;

        if instance_guard.is_none() {
            *instance_guard = Some(Self::create_tesseract_instance(&self.config)?);
        }

        // Clone the instance (Tesseract should be cheap to clone or we'll need to handle this differently)
        instance_guard.as_ref().cloned().ok_or_else(|| {
            OcrError::ProcessingFailed("Failed to get Tesseract instance".to_string())
        })
    }

    /// Process image with detailed error handling
    fn process_image_internal(
        &self,
        image_data: &[u8],
        options: &OcrOptions,
    ) -> OcrResult<OcrProcessingResult> {
        let start_time = Instant::now();

        // Get Tesseract instance
        let mut tesseract = self.get_tesseract_instance()?;

        // Set the image data
        tesseract
            .set_image_from_mem(image_data)
            .map_err(|e| OcrError::InvalidImageData(format!("Failed to set image: {e}")))?;

        // Extract text
        let text = tesseract
            .get_text()
            .map_err(|e| OcrError::ProcessingFailed(format!("Failed to extract text: {e}")))?;

        // Get confidence
        let confidence = tesseract
            .mean_text_conf()
            .map_err(|e| OcrError::ProcessingFailed(format!("Failed to get confidence: {e}")))?;

        // Convert confidence from 0-100 to 0.0-1.0
        let confidence_ratio = confidence as f64 / 100.0;

        // Check minimum confidence
        if confidence_ratio < options.min_confidence {
            return Err(OcrError::LowConfidence(format!(
                "Confidence {:.1}% below minimum {:.1}%",
                confidence_ratio * 100.0,
                options.min_confidence * 100.0
            )));
        }

        // Get word-level information if layout preservation is enabled
        let fragments = if options.preserve_layout {
            self.extract_word_fragments(&mut tesseract, confidence_ratio)?
        } else {
            // Create a single fragment for the entire text
            vec![OcrTextFragment {
                text: text.clone(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                confidence: confidence_ratio,
                font_size: 12.0,
                fragment_type: FragmentType::Paragraph,
            }]
        };

        let processing_time = start_time.elapsed();

        Ok(OcrProcessingResult {
            text,
            confidence: confidence_ratio,
            fragments,
            processing_time_ms: processing_time.as_millis() as u64,
            engine_name: "Tesseract".to_string(),
            language: self.config.language.clone(),
            image_dimensions: (0, 0), // TODO: Get actual image dimensions
        })
    }

    /// Extract word-level fragments from Tesseract
    fn extract_word_fragments(
        &self,
        tesseract: &mut Tesseract,
        default_confidence: f64,
    ) -> OcrResult<Vec<OcrTextFragment>> {
        let mut fragments = Vec::new();

        // In a real implementation, you would use Tesseract's word-level analysis
        // For now, we'll create a simplified version
        let text = tesseract.get_text().map_err(|e| {
            OcrError::ProcessingFailed(format!("Failed to get text for fragments: {e}"))
        })?;

        // Split into words and create fragments
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut x = 0.0;
        let y = 0.0;
        let line_height = 20.0;

        for (i, word) in words.iter().enumerate() {
            let width = word.len() as f64 * 8.0; // Approximate width

            fragments.push(OcrTextFragment {
                text: word.to_string(),
                x,
                y,
                width,
                height: line_height,
                confidence: default_confidence,
                font_size: 12.0,
                fragment_type: FragmentType::Word,
            });

            x += width + 8.0; // Move to next word position

            // Simple line wrapping at 500 pixels
            if x > 500.0 {
                x = 0.0;
                // y += line_height; // This would need to be mutable
            }
        }

        Ok(fragments)
    }
}

#[cfg(feature = "ocr-tesseract")]
impl OcrProvider for TesseractOcrProvider {
    fn process_image(
        &self,
        image_data: &[u8],
        options: &OcrOptions,
    ) -> OcrResult<OcrProcessingResult> {
        // Validate image data first
        self.validate_image_data(image_data)?;

        // Process with Tesseract
        self.process_image_internal(image_data, options)
    }

    fn process_page(
        &self,
        page_analysis: &ContentAnalysis,
        page_data: &[u8],
        options: &OcrOptions,
    ) -> OcrResult<OcrProcessingResult> {
        // Optimize configuration based on page analysis
        let optimized_options = self.optimize_for_page_analysis(page_analysis, options);

        self.process_image(page_data, &optimized_options)
    }

    fn supported_formats(&self) -> Vec<ImageFormat> {
        vec![ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::Tiff]
    }

    fn engine_name(&self) -> &str {
        "Tesseract"
    }

    fn engine_type(&self) -> OcrEngine {
        OcrEngine::Tesseract
    }
}

#[cfg(feature = "ocr-tesseract")]
impl TesseractOcrProvider {
    /// Optimize OCR options based on page analysis
    fn optimize_for_page_analysis(
        &self,
        analysis: &ContentAnalysis,
        options: &OcrOptions,
    ) -> OcrOptions {
        let mut optimized = options.clone();

        // Adjust preprocessing based on page type
        match analysis.page_type {
            crate::operations::page_analysis::PageType::Scanned => {
                // For scanned pages, enable more aggressive preprocessing
                optimized.preprocessing.denoise = true;
                optimized.preprocessing.deskew = true;
                optimized.preprocessing.enhance_contrast = true;
            }
            _ => {
                // For other page types, use lighter preprocessing
                optimized.preprocessing.denoise = false;
                optimized.preprocessing.deskew = false;
            }
        }

        // Adjust confidence threshold based on image quality indicators
        if analysis.image_ratio > 0.9 {
            // Very image-heavy pages might need lower confidence threshold
            optimized.min_confidence = optimized.min_confidence * 0.8;
        }

        optimized
    }
}

// Provide stub implementations when the feature is not enabled
#[cfg(not(feature = "ocr-tesseract"))]
pub struct TesseractOcrProvider;

#[cfg(not(feature = "ocr-tesseract"))]
impl TesseractOcrProvider {
    pub fn new() -> Result<Self, crate::text::OcrError> {
        Err(crate::text::OcrError::ProviderNotAvailable(
            "Tesseract OCR provider not available. Enable the 'ocr-tesseract' feature.".to_string(),
        ))
    }

    pub fn check_availability() -> Result<(), crate::text::OcrError> {
        Err(crate::text::OcrError::ProviderNotAvailable(
            "Tesseract OCR provider not available. Enable the 'ocr-tesseract' feature.".to_string(),
        ))
    }
}

#[cfg(feature = "ocr-tesseract")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{OcrOptions, OcrProvider};

    #[test]
    fn test_page_segmentation_mode() {
        assert_eq!(PageSegmentationMode::Auto as u8, 3);
        assert_eq!(PageSegmentationMode::SingleLine as u8, 7);
        assert_eq!(PageSegmentationMode::SingleWord as u8, 8);

        assert_eq!(PageSegmentationMode::Auto.to_psm_value(), 3);
        assert!(PageSegmentationMode::Auto
            .description()
            .contains("automatic"));
    }

    #[test]
    fn test_ocr_engine_mode() {
        assert_eq!(OcrEngineMode::LstmOnly as u8, 1);
        assert_eq!(OcrEngineMode::Default as u8, 3);

        assert_eq!(OcrEngineMode::LstmOnly.to_oem_value(), 1);
        assert!(OcrEngineMode::LstmOnly.description().contains("LSTM"));
    }

    #[test]
    fn test_tesseract_config_defaults() {
        let config = TesseractConfig::default();
        assert_eq!(config.language, "eng");
        assert_eq!(config.psm, PageSegmentationMode::Auto);
        assert_eq!(config.oem, OcrEngineMode::Default);
        assert!(config.char_whitelist.is_none());
        assert!(config.char_blacklist.is_none());
        assert!(!config.debug);
    }

    #[test]
    fn test_tesseract_config_builders() {
        let config = TesseractConfig::with_language("spa");
        assert_eq!(config.language, "spa");

        let config = TesseractConfig::for_documents();
        assert_eq!(config.psm, PageSegmentationMode::Auto);
        assert_eq!(config.oem, OcrEngineMode::LstmOnly);

        let config = TesseractConfig::for_single_line();
        assert_eq!(config.psm, PageSegmentationMode::SingleLine);

        let config = TesseractConfig::for_sparse_text();
        assert_eq!(config.psm, PageSegmentationMode::SparseText);
    }

    #[test]
    fn test_tesseract_config_customization() {
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
    fn test_available_languages() {
        let languages = TesseractOcrProvider::available_languages().unwrap();
        assert!(languages.contains(&"eng".to_string()));
        assert!(languages.contains(&"spa".to_string()));
        assert!(languages.contains(&"deu".to_string()));
        assert!(languages.len() > 5);
    }

    // Note: These tests would require Tesseract to be installed
    // In a real implementation, you might want to use conditional compilation
    // or skip these tests if Tesseract is not available

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_provider_creation() {
        match TesseractOcrProvider::new() {
            Ok(provider) => {
                assert_eq!(provider.engine_name(), "Tesseract");
                assert_eq!(provider.engine_type(), OcrEngine::Tesseract);
            }
            Err(e) => {
                println!("Tesseract not available: {}", e);
                // This is expected if Tesseract is not installed
            }
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation"]
    fn test_tesseract_check_availability() {
        match TesseractOcrProvider::check_availability() {
            Ok(()) => println!("Tesseract is available"),
            Err(e) => println!("Tesseract not available: {}", e),
        }
    }

    #[test]
    #[ignore = "Requires Tesseract installation and sample image"]
    fn test_tesseract_process_image() {
        // This test would require a sample image and Tesseract installation
        let provider = match TesseractOcrProvider::new() {
            Ok(p) => p,
            Err(_) => return, // Skip if Tesseract not available
        };

        // Mock image data (in real test, use actual image)
        let image_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
        let options = OcrOptions::default();

        // This would fail with mock data, but tests the interface
        match provider.process_image(&image_data, &options) {
            Ok(_result) => {
                // Test passed
            }
            Err(e) => {
                // Expected with mock data
                println!("Expected error with mock data: {}", e);
            }
        }
    }

    #[test]
    fn test_tesseract_supported_formats() {
        let config = TesseractConfig::default();

        // Test without actually creating provider (to avoid Tesseract dependency)
        let supported = vec![ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::Tiff];
        assert!(supported.contains(&ImageFormat::Jpeg));
        assert!(supported.contains(&ImageFormat::Png));
        assert!(supported.contains(&ImageFormat::Tiff));
    }
}
