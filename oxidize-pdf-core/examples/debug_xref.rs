use oxidize_pdf::parser::PdfReader;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Test single file
        let file = &args[1];
        println!("Testing: {file}");

        match PdfReader::open(file) {
            Ok(_) => println!("Success!"),
            Err(e) => {
                println!("Error: {e:?}");
                eprintln!("\nDetailed error: {e:#?}");
            }
        }
    } else {
        // Test all PDFs in test directories
        let test_dirs = vec![
            "../test-suite/external-suites/veraPDF-corpus/ISO 32000-1",
            "../test-suite/external-suites/veraPDF-corpus/ISO 32000-2",
            "../test-suite/external-suites/veraPDF-corpus/TWG test files",
        ];

        let mut total = 0;
        let mut success = 0;
        let mut invalid_xref_count = 0;

        for dir in test_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.extension().map(|e| e.to_str()) == Some(Some("pdf")) {
                            total += 1;

                            match PdfReader::open(&path) {
                                Ok(_) => success += 1,
                                Err(e) => {
                                    if format!("{e:?}").contains("InvalidXRef") {
                                        invalid_xref_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("\nSummary:");
        println!("Total PDFs tested: {total}");
        println!(
            "Success: {} ({:.1}%)",
            success,
            (success as f64 / total as f64 * 100.0)
        );
        println!(
            "InvalidXRef errors: {} ({:.1}%)",
            invalid_xref_count,
            (invalid_xref_count as f64 / total as f64 * 100.0)
        );
    }
}
