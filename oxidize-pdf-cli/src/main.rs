use clap::{Parser, Subcommand};
use oxidize_pdf_core::{Document, Page, Font, Color, Result};
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
    
    /// Future: Merge multiple PDFs (not yet implemented)
    Merge {
        /// Input PDF files
        files: Vec<PathBuf>,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Future: Split a PDF (not yet implemented)
    Split {
        /// Input PDF file
        input: PathBuf,
        
        /// Output prefix
        #[arg(short, long, default_value = "page")]
        prefix: String,
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
    }
    
    Ok(())
}