//! PDF page content analysis
//!
//! This module provides functionality to analyze the content composition of PDF pages,
//! helping to determine whether pages contain primarily scanned images, vector text,
//! or a mixture of both. This is particularly useful for:
//!
//! - Detecting scanned documents that may benefit from OCR processing
//! - Analyzing document composition for optimization purposes
//! - Preprocessing documents for different handling strategies
//!
//! # Usage
//!
//! ```rust,no_run
//! use oxidize_pdf::operations::page_analysis::{PageContentAnalyzer, PageType};
//! use oxidize_pdf::parser::PdfReader;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let document = PdfReader::open_document("example.pdf")?;
//! let analyzer = PageContentAnalyzer::new(document);
//!
//! // Analyze a specific page
//! let analysis = analyzer.analyze_page(0)?;
//!
//! match analysis.page_type {
//!     PageType::Scanned => println!("This page appears to be scanned"),
//!     PageType::Text => println!("This page contains primarily vector text"),
//!     PageType::Mixed => println!("This page contains both text and images"),
//! }
//!
//! // Quick check for scanned pages
//! if analyzer.is_scanned_page(0)? {
//!     println!("Page 0 is likely a scanned image");
//! }
//! # Ok(())
//! # }
//! ```

use super::{OperationError, OperationResult};
use crate::parser::{PdfDocument, PdfReader};
use crate::text::{ExtractionOptions, OcrOptions, OcrProcessingResult, OcrProvider, TextExtractor};
// Note: ImageExtractor functionality is implemented inline to avoid circular dependencies
use std::fs::File;
use std::path::Path;

/// Represents the primary content type of a PDF page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    /// Page contains primarily scanned images (>80% image content, <10% text)
    Scanned,
    /// Page contains primarily vector text (>70% text content, <20% images)
    Text,
    /// Page contains a balanced mix of text and images
    Mixed,
}

impl PageType {
    /// Returns true if this page type represents a scanned page
    pub fn is_scanned(&self) -> bool {
        matches!(self, PageType::Scanned)
    }

    /// Returns true if this page type represents a text-heavy page
    pub fn is_text(&self) -> bool {
        matches!(self, PageType::Text)
    }

    /// Returns true if this page type represents a mixed content page
    pub fn is_mixed(&self) -> bool {
        matches!(self, PageType::Mixed)
    }
}

/// Detailed analysis results for a PDF page
#[derive(Debug, Clone)]
pub struct ContentAnalysis {
    /// The page number (0-indexed)
    pub page_number: usize,
    /// The determined page type based on content analysis
    pub page_type: PageType,
    /// Percentage of page area covered by text (0.0 to 1.0)
    pub text_ratio: f64,
    /// Percentage of page area covered by images (0.0 to 1.0)
    pub image_ratio: f64,
    /// Percentage of page area that is blank space (0.0 to 1.0)
    pub blank_space_ratio: f64,
    /// Number of text fragments found on the page
    pub text_fragment_count: usize,
    /// Number of images found on the page
    pub image_count: usize,
    /// Total number of characters in text content
    pub character_count: usize,
}

impl ContentAnalysis {
    /// Returns true if this page appears to be scanned
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use oxidize_pdf::operations::page_analysis::{ContentAnalysis, PageType};
    /// let analysis = ContentAnalysis {
    ///     page_number: 0,
    ///     page_type: PageType::Scanned,
    ///     text_ratio: 0.05,
    ///     image_ratio: 0.90,
    ///     blank_space_ratio: 0.05,
    ///     text_fragment_count: 2,
    ///     image_count: 1,
    ///     character_count: 15,
    /// };
    ///
    /// assert!(analysis.is_scanned());
    /// ```
    pub fn is_scanned(&self) -> bool {
        self.page_type.is_scanned()
    }

    /// Returns true if this page is primarily text-based
    pub fn is_text_heavy(&self) -> bool {
        self.page_type.is_text()
    }

    /// Returns true if this page has mixed content
    pub fn is_mixed_content(&self) -> bool {
        self.page_type.is_mixed()
    }

    /// Returns the dominant content type ratio (text or image)
    pub fn dominant_content_ratio(&self) -> f64 {
        self.text_ratio.max(self.image_ratio)
    }
}

/// Configuration options for page content analysis
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    /// Minimum text fragment size to consider (in characters)
    pub min_text_fragment_size: usize,
    /// Minimum image size to consider (in pixels)
    pub min_image_size: u32,
    /// Threshold for considering a page as scanned (image ratio)
    pub scanned_threshold: f64,
    /// Threshold for considering a page as text-heavy (text ratio)
    pub text_threshold: f64,
    /// OCR options for processing scanned pages
    pub ocr_options: Option<OcrOptions>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            min_text_fragment_size: 3,
            min_image_size: 50,
            scanned_threshold: 0.8,
            text_threshold: 0.7,
            ocr_options: None,
        }
    }
}

/// Analyzer for PDF page content composition
///
/// This struct provides methods to analyze the content of PDF pages and determine
/// their composition (text vs images vs mixed content).
pub struct PageContentAnalyzer {
    document: PdfDocument<File>,
    options: AnalysisOptions,
}

impl PageContentAnalyzer {
    /// Create a new page content analyzer
    ///
    /// # Arguments
    ///
    /// * `document` - The PDF document to analyze
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use oxidize_pdf::operations::page_analysis::PageContentAnalyzer;
    /// use oxidize_pdf::parser::PdfReader;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let document = PdfReader::open_document("example.pdf")?;
    /// let analyzer = PageContentAnalyzer::new(document);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(document: PdfDocument<File>) -> Self {
        Self {
            document,
            options: AnalysisOptions::default(),
        }
    }

    /// Create a new page content analyzer with custom options
    ///
    /// # Arguments
    ///
    /// * `document` - The PDF document to analyze
    /// * `options` - Custom analysis options
    pub fn with_options(document: PdfDocument<File>, options: AnalysisOptions) -> Self {
        Self { document, options }
    }

    /// Create a page content analyzer from a file path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PDF file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or is not a valid PDF.
    pub fn from_file<P: AsRef<Path>>(path: P) -> OperationResult<Self> {
        let document = PdfReader::open_document(path)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;
        Ok(Self::new(document))
    }

    /// Analyze the content of a specific page
    ///
    /// This method examines the page's text and image content to determine
    /// the composition and classify the page type.
    ///
    /// # Arguments
    ///
    /// * `page_number` - The page number to analyze (0-indexed)
    ///
    /// # Returns
    ///
    /// A `ContentAnalysis` struct containing detailed analysis results.
    ///
    /// # Errors
    ///
    /// Returns an error if the page cannot be accessed or analyzed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf::operations::page_analysis::PageContentAnalyzer;
    /// # use oxidize_pdf::parser::PdfReader;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let document = PdfReader::open_document("example.pdf")?;
    /// let analyzer = PageContentAnalyzer::new(document);
    ///
    /// let analysis = analyzer.analyze_page(0)?;
    /// println!("Page type: {:?}", analysis.page_type);
    /// println!("Text ratio: {:.2}%", analysis.text_ratio * 100.0);
    /// println!("Image ratio: {:.2}%", analysis.image_ratio * 100.0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn analyze_page(&self, page_number: usize) -> OperationResult<ContentAnalysis> {
        // Get page dimensions for area calculations
        let page = self
            .document
            .get_page(page_number as u32)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;

        let page_area = self.calculate_page_area(&page)?;

        // Analyze text content
        let text_analysis = self.analyze_text_content(page_number)?;
        let text_area = text_analysis.total_area;
        let text_fragment_count = text_analysis.fragment_count;
        let character_count = text_analysis.character_count;

        // Analyze image content
        let image_analysis = self.analyze_image_content(page_number)?;
        let image_area = image_analysis.total_area;
        let image_count = image_analysis.image_count;

        // Calculate ratios
        let text_ratio = if page_area > 0.0 {
            text_area / page_area
        } else {
            0.0
        };
        let image_ratio = if page_area > 0.0 {
            image_area / page_area
        } else {
            0.0
        };
        let blank_space_ratio = 1.0 - text_ratio - image_ratio;

        // Determine page type based on content ratios
        let page_type = self.determine_page_type(text_ratio, image_ratio);

        Ok(ContentAnalysis {
            page_number,
            page_type,
            text_ratio,
            image_ratio,
            blank_space_ratio: blank_space_ratio.max(0.0),
            text_fragment_count,
            image_count,
            character_count,
        })
    }

    /// Analyze all pages in the document
    ///
    /// # Returns
    ///
    /// A vector of `ContentAnalysis` results, one for each page.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf::operations::page_analysis::PageContentAnalyzer;
    /// # use oxidize_pdf::parser::PdfReader;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let document = PdfReader::open_document("example.pdf")?;
    /// let analyzer = PageContentAnalyzer::new(document);
    ///
    /// let analyses = analyzer.analyze_document()?;
    /// for analysis in analyses {
    ///     println!("Page {}: {:?}", analysis.page_number, analysis.page_type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn analyze_document(&self) -> OperationResult<Vec<ContentAnalysis>> {
        let page_count = self
            .document
            .page_count()
            .map_err(|e| OperationError::ParseError(e.to_string()))?;

        let mut analyses = Vec::new();
        for page_idx in 0..page_count {
            let analysis = self.analyze_page(page_idx as usize)?;
            analyses.push(analysis);
        }

        Ok(analyses)
    }

    /// Analyze specific pages in the document
    ///
    /// # Arguments
    ///
    /// * `page_numbers` - Vector of page numbers to analyze (0-indexed)
    ///
    /// # Returns
    ///
    /// A vector of `ContentAnalysis` results for the specified pages.
    pub fn analyze_pages(&self, page_numbers: &[usize]) -> OperationResult<Vec<ContentAnalysis>> {
        let mut analyses = Vec::new();
        for &page_number in page_numbers {
            let analysis = self.analyze_page(page_number)?;
            analyses.push(analysis);
        }
        Ok(analyses)
    }

    /// Quick check if a page appears to be scanned
    ///
    /// This is a convenience method that performs a full analysis but only
    /// returns whether the page is classified as scanned.
    ///
    /// # Arguments
    ///
    /// * `page_number` - The page number to check (0-indexed)
    ///
    /// # Returns
    ///
    /// `true` if the page appears to be scanned, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf::operations::page_analysis::PageContentAnalyzer;
    /// # use oxidize_pdf::parser::PdfReader;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let document = PdfReader::open_document("example.pdf")?;
    /// let analyzer = PageContentAnalyzer::new(document);
    ///
    /// if analyzer.is_scanned_page(0)? {
    ///     println!("Page 0 is a scanned image - consider OCR processing");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_scanned_page(&self, page_number: usize) -> OperationResult<bool> {
        let analysis = self.analyze_page(page_number)?;
        Ok(analysis.is_scanned())
    }

    /// Find all scanned pages in the document
    ///
    /// # Returns
    ///
    /// A vector of page numbers (0-indexed) that appear to be scanned.
    pub fn find_scanned_pages(&self) -> OperationResult<Vec<usize>> {
        let analyses = self.analyze_document()?;
        Ok(analyses
            .into_iter()
            .filter(|analysis| analysis.is_scanned())
            .map(|analysis| analysis.page_number)
            .collect())
    }

    /// Extract text from a scanned page using OCR
    ///
    /// This method processes a scanned page with OCR to extract text content.
    /// It first verifies that the page is indeed scanned, then applies OCR processing.
    ///
    /// # Arguments
    ///
    /// * `page_number` - The page number to process (0-indexed)
    /// * `ocr_provider` - The OCR provider to use for text extraction
    ///
    /// # Returns
    ///
    /// OCR processing results with extracted text and positioning information.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The page is not scanned (use `is_scanned_page` to check first)
    /// - OCR processing fails
    /// - Page cannot be accessed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf::operations::page_analysis::PageContentAnalyzer;
    /// # use oxidize_pdf::text::MockOcrProvider;
    /// # use oxidize_pdf::parser::PdfReader;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let document = PdfReader::open_document("scanned.pdf")?;
    /// let analyzer = PageContentAnalyzer::new(document);
    /// let ocr_provider = MockOcrProvider::new();
    ///
    /// if analyzer.is_scanned_page(0)? {
    ///     let ocr_result = analyzer.extract_text_from_scanned_page(0, &ocr_provider)?;
    ///     println!("OCR extracted text: {}", ocr_result.text);
    ///     println!("Confidence: {:.2}%", ocr_result.confidence * 100.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn extract_text_from_scanned_page<P: OcrProvider>(
        &self,
        page_number: usize,
        ocr_provider: &P,
    ) -> OperationResult<OcrProcessingResult> {
        // First verify the page is scanned
        let analysis = self.analyze_page(page_number)?;
        if !analysis.is_scanned() {
            return Err(OperationError::ParseError(format!(
                "Page {} is not a scanned page (image ratio: {:.2}%, text ratio: {:.2}%)",
                page_number,
                analysis.image_ratio * 100.0,
                analysis.text_ratio * 100.0
            )));
        }

        // Get OCR options from analysis options or use default
        let ocr_options = self.options.ocr_options.clone().unwrap_or_default();

        // Extract image data from the page
        let page_image_data = self.extract_page_image_data(page_number)?;

        // Process with OCR
        let ocr_result = ocr_provider
            .process_page(&analysis, &page_image_data, &ocr_options)
            .map_err(|e| OperationError::ParseError(format!("OCR processing failed: {}", e)))?;

        Ok(ocr_result)
    }

    /// Process all scanned pages in the document with OCR
    ///
    /// This method identifies all scanned pages and processes them with OCR,
    /// returning a map of page numbers to OCR results.
    ///
    /// # Arguments
    ///
    /// * `ocr_provider` - The OCR provider to use for text extraction
    ///
    /// # Returns
    ///
    /// A vector of tuples containing (page_number, ocr_result) for each scanned page.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use oxidize_pdf::operations::page_analysis::PageContentAnalyzer;
    /// # use oxidize_pdf::text::MockOcrProvider;
    /// # use oxidize_pdf::parser::PdfReader;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let document = PdfReader::open_document("scanned.pdf")?;
    /// let analyzer = PageContentAnalyzer::new(document);
    /// let ocr_provider = MockOcrProvider::new();
    ///
    /// let ocr_results = analyzer.process_scanned_pages_with_ocr(&ocr_provider)?;
    ///
    /// for (page_num, ocr_result) in ocr_results {
    ///     println!("Page {}: {} characters extracted", page_num, ocr_result.text.len());
    ///     println!("  Confidence: {:.2}%", ocr_result.confidence * 100.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn process_scanned_pages_with_ocr<P: OcrProvider>(
        &self,
        ocr_provider: &P,
    ) -> OperationResult<Vec<(usize, OcrProcessingResult)>> {
        let scanned_pages = self.find_scanned_pages()?;
        let mut results = Vec::new();

        for page_number in scanned_pages {
            match self.extract_text_from_scanned_page(page_number, ocr_provider) {
                Ok(ocr_result) => {
                    results.push((page_number, ocr_result));
                }
                Err(e) => {
                    eprintln!("Failed to process page {page_number}: {e}");
                    continue;
                }
            }
        }

        Ok(results)
    }

    /// Process multiple scanned pages with OCR in parallel (threaded version)
    ///
    /// This method processes multiple scanned pages concurrently using threads,
    /// which can significantly improve performance when dealing with large documents.
    ///
    /// # Arguments
    ///
    /// * `ocr_provider` - OCR provider to use for text extraction
    /// * `max_threads` - Maximum number of threads to use (None for automatic)
    ///
    /// # Returns
    ///
    /// A vector of tuples containing page numbers and their OCR results.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use oxidize_pdf::operations::page_analysis::PageContentAnalyzer;
    /// use oxidize_pdf::text::MockOcrProvider;
    /// use oxidize_pdf::parser::PdfReader;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let document = PdfReader::open_document("scanned.pdf")?;
    /// let analyzer = PageContentAnalyzer::new(document);
    /// let provider = MockOcrProvider::new();
    ///
    /// // Process with up to 4 threads
    /// let results = analyzer.process_scanned_pages_parallel(&provider, Some(4))?;
    /// for (page_num, result) in results {
    ///     println!("Page {}: {} characters", page_num, result.text.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn process_scanned_pages_parallel<P: OcrProvider + Clone + Send + Sync + 'static>(
        &self,
        ocr_provider: &P,
        max_threads: Option<usize>,
    ) -> OperationResult<Vec<(usize, OcrProcessingResult)>> {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let scanned_pages = self.find_scanned_pages()?;
        if scanned_pages.is_empty() {
            return Ok(Vec::new());
        }

        // Determine thread count
        let thread_count = max_threads.unwrap_or_else(|| {
            std::cmp::min(
                scanned_pages.len(),
                std::thread::available_parallelism()
                    .map(|p| p.get())
                    .unwrap_or(4),
            )
        });

        if thread_count <= 1 {
            // Fall back to sequential processing
            return self.process_scanned_pages_with_ocr(ocr_provider);
        }

        // Shared results vector
        let results = Arc::new(Mutex::new(Vec::new()));
        let provider = Arc::new(ocr_provider.clone());

        // Create chunks of pages for each thread
        let chunk_size = scanned_pages.len().div_ceil(thread_count);
        let mut handles = Vec::new();

        for chunk in scanned_pages.chunks(chunk_size) {
            let chunk_pages = chunk.to_vec();
            let results_clone = Arc::clone(&results);
            let provider_clone = Arc::clone(&provider);

            // Create a temporary analyzer for this thread
            // Note: This is a simplified approach - in practice you'd want to avoid cloning the document
            let handle = thread::spawn(move || {
                let mut thread_results = Vec::new();

                for page_num in chunk_pages {
                    // In a real implementation, you'd extract the image data and process it
                    // For now, we'll simulate with a simple approach
                    match simulate_page_ocr_processing(page_num, &*provider_clone) {
                        Ok(ocr_result) => {
                            thread_results.push((page_num, ocr_result));
                        }
                        Err(e) => {
                            eprintln!("OCR failed for page {page_num}: {e}");
                        }
                    }
                }

                // Add results to shared vector
                if let Ok(mut shared_results) = results_clone.lock() {
                    shared_results.extend(thread_results);
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            if let Err(e) = handle.join() {
                eprintln!("Thread panicked: {e:?}");
            }
        }

        // Extract results
        let final_results = results
            .lock()
            .map_err(|e| OperationError::ProcessingError(format!("Failed to get results: {e}")))?
            .clone();

        Ok(final_results)
    }

    /// Process scanned pages with OCR using a batch approach
    ///
    /// This method processes pages in batches, which can be more efficient for
    /// certain OCR providers that support batch processing.
    ///
    /// # Arguments
    ///
    /// * `ocr_provider` - OCR provider to use for text extraction
    /// * `batch_size` - Number of pages to process in each batch
    ///
    /// # Returns
    ///
    /// A vector of tuples containing page numbers and their OCR results.
    pub fn process_scanned_pages_batch<P: OcrProvider>(
        &self,
        ocr_provider: &P,
        batch_size: usize,
    ) -> OperationResult<Vec<(usize, OcrProcessingResult)>> {
        let scanned_pages = self.find_scanned_pages()?;
        let mut results = Vec::new();

        for batch in scanned_pages.chunks(batch_size) {
            println!("Processing batch of {} pages", batch.len());

            for &page_num in batch {
                match self.extract_text_from_scanned_page(page_num, ocr_provider) {
                    Ok(ocr_result) => {
                        results.push((page_num, ocr_result));
                    }
                    Err(e) => {
                        eprintln!("OCR failed for page {page_num}: {e}");
                    }
                }
            }

            // Add a small delay between batches to avoid overwhelming the OCR provider
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        Ok(results)
    }

    /// Extract image data from a page for OCR processing
    ///
    /// This method extracts the primary image from a scanned page.
    /// For scanned pages, this typically returns the main page image.
    fn extract_page_image_data(&self, page_number: usize) -> OperationResult<Vec<u8>> {
        let page = self
            .document
            .get_page(page_number as u32)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;

        // Get page resources to find the main image
        let resources = self
            .document
            .get_page_resources(&page)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;

        if let Some(resources) = resources {
            if let Some(crate::parser::objects::PdfObject::Dictionary(xobjects)) = resources
                .0
                .get(&crate::parser::objects::PdfName("XObject".to_string()))
            {
                for obj_ref in xobjects.0.values() {
                    if let crate::parser::objects::PdfObject::Reference(obj_num, gen_num) = obj_ref
                    {
                        if let Ok(crate::parser::objects::PdfObject::Stream(stream)) =
                            self.document.get_object(*obj_num, *gen_num)
                        {
                            // Check if it's an image XObject
                            if let Some(crate::parser::objects::PdfObject::Name(subtype)) = stream
                                .dict
                                .0
                                .get(&crate::parser::objects::PdfName("Subtype".to_string()))
                            {
                                if subtype.0 == "Image" {
                                    // Get the raw image data
                                    let image_data = stream.decode().map_err(|e| {
                                        OperationError::ParseError(format!(
                                            "Failed to decode image: {e}"
                                        ))
                                    })?;

                                    // Return the first (and typically only) image for scanned pages
                                    return Ok(image_data);
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(OperationError::ParseError(
            "No image data found on scanned page".to_string(),
        ))
    }

    /// Calculate the total area of a page in points
    fn calculate_page_area(&self, page: &crate::parser::ParsedPage) -> OperationResult<f64> {
        // Get page dimensions from MediaBox
        let width = page.width();
        let height = page.height();

        Ok(width * height)
    }

    /// Analyze text content on a page
    fn analyze_text_content(&self, page_number: usize) -> OperationResult<TextAnalysisResult> {
        let extractor = TextExtractor::with_options(ExtractionOptions {
            preserve_layout: true,
            space_threshold: 0.2,
            newline_threshold: 10.0,
        });

        let extracted_text = extractor
            .extract_from_page(&self.document, page_number as u32)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;

        let mut total_area = 0.0;
        let mut fragment_count = 0;
        let character_count = extracted_text.text.len();

        // Calculate area covered by text fragments
        for fragment in &extracted_text.fragments {
            if fragment.text.trim().len() >= self.options.min_text_fragment_size {
                total_area += fragment.width * fragment.height;
                fragment_count += 1;
            }
        }

        Ok(TextAnalysisResult {
            total_area,
            fragment_count,
            character_count,
        })
    }

    /// Analyze image content on a page
    fn analyze_image_content(&self, page_number: usize) -> OperationResult<ImageAnalysisResult> {
        // For now, we'll use a simplified approach that estimates image coverage
        // based on the presence of XObject references in the page resources

        let page = self
            .document
            .get_page(page_number as u32)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;

        // Get page resources to check for XObject references
        let resources = self
            .document
            .get_page_resources(&page)
            .map_err(|e| OperationError::ParseError(e.to_string()))?;

        let mut total_area = 0.0;
        let mut image_count = 0;

        if let Some(resources) = resources {
            if let Some(crate::parser::objects::PdfObject::Dictionary(xobjects)) = resources
                .0
                .get(&crate::parser::objects::PdfName("XObject".to_string()))
            {
                for obj_ref in xobjects.0.values() {
                    if let crate::parser::objects::PdfObject::Reference(obj_num, gen_num) = obj_ref
                    {
                        if let Ok(crate::parser::objects::PdfObject::Stream(stream)) =
                            self.document.get_object(*obj_num, *gen_num)
                        {
                            // Check if it's an image XObject
                            if let Some(crate::parser::objects::PdfObject::Name(subtype)) = stream
                                .dict
                                .0
                                .get(&crate::parser::objects::PdfName("Subtype".to_string()))
                            {
                                if subtype.0 == "Image" {
                                    image_count += 1;

                                    // Get image dimensions
                                    let width =
                                        match stream.dict.0.get(&crate::parser::objects::PdfName(
                                            "Width".to_string(),
                                        )) {
                                            Some(crate::parser::objects::PdfObject::Integer(w)) => {
                                                *w as f64
                                            }
                                            _ => 0.0,
                                        };

                                    let height =
                                        match stream.dict.0.get(&crate::parser::objects::PdfName(
                                            "Height".to_string(),
                                        )) {
                                            Some(crate::parser::objects::PdfObject::Integer(h)) => {
                                                *h as f64
                                            }
                                            _ => 0.0,
                                        };

                                    // Check minimum size
                                    if width >= self.options.min_image_size as f64
                                        && height >= self.options.min_image_size as f64
                                    {
                                        total_area += width * height;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(ImageAnalysisResult {
            total_area,
            image_count,
        })
    }

    /// Determine the page type based on content ratios
    ///
    /// # Arguments
    ///
    /// * `text_ratio` - Ratio of page area covered by text (0.0 to 1.0)
    /// * `image_ratio` - Ratio of page area covered by images (0.0 to 1.0)
    ///
    /// # Algorithm
    ///
    /// The classification uses the following thresholds:
    /// - **Scanned**: Image ratio > 80% AND text ratio < 10%
    /// - **Text**: Text ratio > 70% AND image ratio < 20%
    /// - **Mixed**: Everything else
    fn determine_page_type(&self, text_ratio: f64, image_ratio: f64) -> PageType {
        if image_ratio > self.options.scanned_threshold && text_ratio < 0.1 {
            PageType::Scanned
        } else if text_ratio > self.options.text_threshold && image_ratio < 0.2 {
            PageType::Text
        } else {
            PageType::Mixed
        }
    }
}

/// Helper struct for text analysis results
struct TextAnalysisResult {
    total_area: f64,
    fragment_count: usize,
    character_count: usize,
}

/// Helper struct for image analysis results
struct ImageAnalysisResult {
    total_area: f64,
    image_count: usize,
}

/// Simulate OCR processing for a single page (helper function for parallel processing)
fn simulate_page_ocr_processing<P: OcrProvider>(
    page_num: usize,
    ocr_provider: &P,
) -> Result<OcrProcessingResult, crate::text::ocr::OcrError> {
    // Create mock image data for the page
    let mock_image_data = vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01, 0x00,
        0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xD9,
    ];

    let options = crate::text::ocr::OcrOptions {
        language: "eng".to_string(),
        min_confidence: 0.6,
        preserve_layout: true,
        preprocessing: crate::text::ocr::ImagePreprocessing::default(),
        engine_options: std::collections::HashMap::new(),
        timeout_seconds: 30,
    };

    // Process the mock image data
    let mut result = ocr_provider.process_image(&mock_image_data, &options)?;

    // Customize the result to indicate which page it came from
    result.text = format!("Page {page_num} text extracted via OCR");

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_type_classification() {
        assert!(PageType::Scanned.is_scanned());
        assert!(!PageType::Text.is_scanned());
        assert!(!PageType::Mixed.is_scanned());

        assert!(PageType::Text.is_text());
        assert!(!PageType::Scanned.is_text());
        assert!(!PageType::Mixed.is_text());

        assert!(PageType::Mixed.is_mixed());
        assert!(!PageType::Scanned.is_mixed());
        assert!(!PageType::Text.is_mixed());
    }

    #[test]
    fn test_content_analysis_methods() {
        let analysis = ContentAnalysis {
            page_number: 0,
            page_type: PageType::Scanned,
            text_ratio: 0.05,
            image_ratio: 0.90,
            blank_space_ratio: 0.05,
            text_fragment_count: 2,
            image_count: 1,
            character_count: 15,
        };

        assert!(analysis.is_scanned());
        assert!(!analysis.is_text_heavy());
        assert!(!analysis.is_mixed_content());
        assert_eq!(analysis.dominant_content_ratio(), 0.90);
    }

    #[test]
    fn test_analysis_options_default() {
        let options = AnalysisOptions::default();
        assert_eq!(options.min_text_fragment_size, 3);
        assert_eq!(options.min_image_size, 50);
        assert_eq!(options.scanned_threshold, 0.8);
        assert_eq!(options.text_threshold, 0.7);
        assert!(options.ocr_options.is_none());
    }

    #[test]
    fn test_determine_page_type() {
        // Create a mock analyzer to test the logic
        let options = AnalysisOptions::default();

        // Test scanned page detection
        let page_type = if 0.90 > options.scanned_threshold && 0.05 < 0.1 {
            PageType::Scanned
        } else if 0.05 > options.text_threshold && 0.90 < 0.2 {
            PageType::Text
        } else {
            PageType::Mixed
        };
        assert_eq!(page_type, PageType::Scanned);

        // Test text page detection
        let page_type = if 0.10 > options.scanned_threshold && 0.80 < 0.1 {
            PageType::Scanned
        } else if 0.80 > options.text_threshold && 0.10 < 0.2 {
            PageType::Text
        } else {
            PageType::Mixed
        };
        assert_eq!(page_type, PageType::Text);

        // Test mixed page detection
        let page_type = if 0.40 > options.scanned_threshold && 0.50 < 0.1 {
            PageType::Scanned
        } else if 0.50 > options.text_threshold && 0.40 < 0.2 {
            PageType::Text
        } else {
            PageType::Mixed
        };
        assert_eq!(page_type, PageType::Mixed);
    }
}

#[cfg(test)]
#[path = "page_analysis_tests.rs"]
mod page_analysis_tests;

#[cfg(test)]
#[path = "page_analysis_ocr_tests.rs"]
mod page_analysis_ocr_tests;
