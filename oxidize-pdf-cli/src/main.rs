use anyhow::Result;
use clap::{Parser, Subcommand};
use oxidize_pdf::operations::{rotate_pdf_pages, PageRange, RotateOptions, RotationAngle};
use oxidize_pdf::text::{ExtractionOptions, TextExtractor};
use oxidize_pdf::{Color, Document, Font, Page, PdfReader};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "oxidizepdf",
    about = "A native Rust PDF processing tool",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a simple PDF with text
    Create {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Text to include in the PDF
        #[arg(short, long)]
        text: String,
    },

    /// Generate a demo PDF with graphics
    Demo {
        /// Output file path  
        #[arg(short, long, default_value = "demo.pdf")]
        output: PathBuf,
    },

    /// Merge multiple PDFs into one
    Merge {
        /// Input PDF files
        files: Vec<PathBuf>,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Page ranges for each file (e.g., "1-5,all,2,4-6")
        #[arg(short, long)]
        pages: Option<Vec<String>>,
    },

    /// Split a PDF into multiple files
    Split {
        /// Input PDF file
        input: PathBuf,

        /// Output pattern (use {} for page number)
        #[arg(short = 'p', long, default_value = "page_{}.pdf")]
        pattern: String,

        /// Split mode: pages, ranges, or at specific pages
        #[arg(short, long, default_value = "pages")]
        mode: String,

        /// Page specification (depends on mode)
        #[arg(long)]
        spec: Option<String>,
    },

    /// Rotate pages in a PDF
    Rotate {
        /// Input PDF file
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Rotation angle (90, 180, 270)
        #[arg(short, long, default_value = "90")]
        angle: i32,

        /// Pages to rotate (e.g., "all", "1,3,5", "2-6")
        #[arg(short = 'p', long, default_value = "all")]
        pages: String,
    },

    /// Get information about a PDF file
    Info {
        /// Input PDF file
        input: PathBuf,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Extract text from a PDF file
    ExtractText {
        /// Input PDF file
        input: PathBuf,

        /// Output text file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Page number to extract (0-based, extracts all if not specified)
        #[arg(short = 'p', long)]
        page: Option<u32>,

        /// Preserve layout
        #[arg(short = 'l', long)]
        preserve_layout: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { output, text } => {
            let mut doc = Document::new();
            let mut page = Page::a4();

            page.text()
                .set_font(Font::Helvetica, 24.0)
                .at(50.0, 750.0)
                .write(&text)?;

            doc.add_page(page);
            doc.save(output)?;

            println!("PDF created successfully!");
        }

        Commands::Demo { output } => {
            let mut doc = Document::new();
            let mut page = Page::a4();

            // Add graphics
            page.graphics()
                .set_stroke_color(Color::red())
                .set_line_width(2.0)
                .rect(50.0, 50.0, 200.0, 100.0)
                .stroke()
                .set_fill_color(Color::blue())
                .circle(300.0, 400.0, 50.0)
                .fill();

            // Add text
            page.text()
                .set_font(Font::HelveticaBold, 36.0)
                .at(100.0, 700.0)
                .write("oxidizePdf Demo")?
                .set_font(Font::Helvetica, 16.0)
                .at(100.0, 650.0)
                .write("Native Rust PDF Generation")?;

            doc.add_page(page);
            doc.set_title("oxidizePdf Demo");
            doc.set_author("oxidizePdf CLI");
            doc.save(output)?;

            println!("Demo PDF created successfully!");
        }

        Commands::Merge { .. } => {
            eprintln!("PDF merge functionality coming in Q2 2025");
            eprintln!("This will require implementing the native PDF parser first");
        }

        Commands::Split { .. } => {
            eprintln!("PDF split functionality coming in Q2 2025");
            eprintln!("This will require implementing the native PDF parser first");
        }

        Commands::Info { input, detailed } => {
            match PdfReader::open(&input) {
                Ok(mut reader) => {
                    println!("PDF Information for: {}", input.display());
                    println!("==========================================");

                    // Basic info
                    let version = reader.version();
                    println!("PDF Version: {}", version.to_string());

                    // Try to get metadata
                    match reader.metadata() {
                        Ok(metadata) => {
                            if let Some(title) = &metadata.title {
                                println!("Title: {}", title);
                            }
                            if let Some(author) = &metadata.author {
                                println!("Author: {}", author);
                            }
                            if let Some(subject) = &metadata.subject {
                                println!("Subject: {}", subject);
                            }
                            if let Some(creator) = &metadata.creator {
                                println!("Creator: {}", creator);
                            }
                            if let Some(producer) = &metadata.producer {
                                println!("Producer: {}", producer);
                            }
                            if let Some(page_count) = metadata.page_count {
                                println!("Pages: {}", page_count);
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Could not read metadata: {}", e);
                        }
                    }

                    if detailed {
                        println!("\nDetailed Information:");
                        println!("--------------------");

                        // Try to get catalog info
                        match reader.catalog() {
                            Ok(catalog) => {
                                if let Some(catalog_type) = catalog.get_type() {
                                    println!("Catalog Type: {}", catalog_type);
                                }

                                // Check for common catalog entries
                                if catalog.contains_key("ViewerPreferences") {
                                    println!("Has Viewer Preferences: Yes");
                                }
                                if catalog.contains_key("Names") {
                                    println!("Has Names Dictionary: Yes");
                                }
                                if catalog.contains_key("Outlines") {
                                    println!("Has Outlines (Bookmarks): Yes");
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Could not read catalog: {}", e);
                            }
                        }

                        // Show page information
                        match reader.page_count() {
                            Ok(count) if count > 0 => {
                                println!("\nPage Information:");
                                println!("-----------------");

                                // Show first few pages
                                let pages_to_show = std::cmp::min(3, count);
                                for i in 0..pages_to_show {
                                    match reader.get_page(i) {
                                        Ok(page) => {
                                            println!(
                                                "Page {}: {:.0}x{:.0} pts",
                                                i + 1,
                                                page.width(),
                                                page.height()
                                            );
                                        }
                                        Err(_) => {
                                            println!("Page {}: [Could not read]", i + 1);
                                        }
                                    }
                                }

                                if count > pages_to_show {
                                    println!("... and {} more pages", count - pages_to_show);
                                }
                            }
                            _ => {}
                        }
                    }

                    println!("\n✓ PDF parsed successfully!");
                }
                Err(e) => {
                    eprintln!("Error: Failed to parse PDF: {}", e);
                    eprintln!("\nNote: The PDF parser is currently in early development.");
                    eprintln!("Some PDF features may not be supported yet.");
                    std::process::exit(1);
                }
            }
        }

        Commands::Rotate {
            input,
            output,
            angle,
            pages,
        } => {
            // Parse rotation angle
            let rotation = RotationAngle::from_degrees(angle).unwrap_or_else(|e| {
                eprintln!("Error: {}. Valid angles are 90, 180, 270", e);
                std::process::exit(1);
            });

            // Parse page range
            let page_range = PageRange::parse(&pages).unwrap_or_else(|e| {
                eprintln!("Error parsing page range '{}': {}", pages, e);
                std::process::exit(1);
            });

            let options = RotateOptions {
                pages: page_range,
                angle: rotation,
                preserve_page_size: false,
            };

            match rotate_pdf_pages(&input, &output, options) {
                Ok(_) => {
                    println!(
                        "✓ Successfully rotated pages {} degrees in {}",
                        angle,
                        output.display()
                    );
                }
                Err(e) => {
                    eprintln!("Error rotating PDF: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::ExtractText {
            input,
            output,
            page,
            preserve_layout,
        } => {
            let document = PdfReader::open_document(&input)
                .map_err(|e| anyhow::anyhow!("Failed to open PDF: {}", e))?;

            let options = ExtractionOptions {
                preserve_layout,
                ..Default::default()
            };

            let extractor = TextExtractor::with_options(options);

            // Extract text from specific page or all pages
            let extracted_text = if let Some(page_num) = page {
                vec![extractor
                    .extract_from_page(&document, page_num)
                    .map_err(|e| {
                        anyhow::anyhow!("Failed to extract text from page {}: {}", page_num, e)
                    })?]
            } else {
                extractor
                    .extract_from_document(&document)
                    .map_err(|e| anyhow::anyhow!("Failed to extract text: {}", e))?
            };

            // Combine all extracted text
            let full_text = extracted_text
                .iter()
                .map(|et| et.text.as_str())
                .collect::<Vec<_>>()
                .join("\n\n");

            // Write to output file or stdout
            if let Some(output_path) = output {
                std::fs::write(&output_path, &full_text)
                    .map_err(|e| anyhow::anyhow!("Failed to write output file: {}", e))?;
                println!("✓ Text extracted to: {}", output_path.display());
            } else {
                println!("{}", full_text);
            }
        }
    }

    Ok(())
}
