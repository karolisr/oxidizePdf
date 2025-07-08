//! Integration tests for PDF operations

use oxidize_pdf_core::{Document, Page, Font, Result};
use oxidize_pdf_core::operations::{
    split_into_pages, merge_pdf_files, rotate_all_pages,
    PageRange, RotationAngle
};
use std::fs;
use std::path::Path;

#[test]
fn test_split_merge_rotate_workflow() -> Result<()> {
    // Create test directory
    let test_dir = "test_output";
    fs::create_dir_all(test_dir).ok();
    
    // Create a test PDF with 3 pages
    let test_pdf = format!("{}/test_multi.pdf", test_dir);
    create_test_pdf(&test_pdf, 3)?;
    
    // Test 1: Split into individual pages
    let split_pattern = format!("{}/split_page_{}.pdf", test_dir);
    let split_files = split_into_pages(&test_pdf, &split_pattern)?;
    assert_eq!(split_files.len(), 3);
    
    // Verify split files exist
    for file in &split_files {
        assert!(file.exists(), "Split file {:?} should exist", file);
    }
    
    // Test 2: Merge the split files back
    let merged_pdf = format!("{}/merged.pdf", test_dir);
    merge_pdf_files(&split_files, &merged_pdf)?;
    assert!(Path::new(&merged_pdf).exists());
    
    // Test 3: Rotate all pages
    let rotated_pdf = format!("{}/rotated.pdf", test_dir);
    rotate_all_pages(&merged_pdf, &rotated_pdf, RotationAngle::Clockwise90)?;
    assert!(Path::new(&rotated_pdf).exists());
    
    // Clean up
    fs::remove_dir_all(test_dir).ok();
    
    Ok(())
}

#[test]
fn test_page_range_parsing() {
    // Test single page
    let range = PageRange::parse("1").unwrap();
    assert_eq!(range.get_indices(10).unwrap(), vec![0]);
    
    // Test range
    let range = PageRange::parse("2-5").unwrap();
    assert_eq!(range.get_indices(10).unwrap(), vec![1, 2, 3, 4]);
    
    // Test multiple pages
    let range = PageRange::parse("1,3,5").unwrap();
    assert_eq!(range.get_indices(10).unwrap(), vec![0, 2, 4]);
    
    // Test all pages
    let range = PageRange::parse("all").unwrap();
    assert_eq!(range.get_indices(3).unwrap(), vec![0, 1, 2]);
    
    // Test reverse range
    let range = PageRange::parse("5-2").unwrap();
    assert_eq!(range.get_indices(10).unwrap(), vec![4, 3, 2, 1]);
}

#[test]
fn test_rotation_angles() {
    assert_eq!(RotationAngle::from_degrees(0).unwrap(), RotationAngle::None);
    assert_eq!(RotationAngle::from_degrees(90).unwrap(), RotationAngle::Clockwise90);
    assert_eq!(RotationAngle::from_degrees(180).unwrap(), RotationAngle::Rotate180);
    assert_eq!(RotationAngle::from_degrees(270).unwrap(), RotationAngle::Clockwise270);
    
    // Test normalization
    assert_eq!(RotationAngle::from_degrees(360).unwrap(), RotationAngle::None);
    assert_eq!(RotationAngle::from_degrees(-90).unwrap(), RotationAngle::Clockwise270);
    
    // Test invalid angles
    assert!(RotationAngle::from_degrees(45).is_err());
}

// Helper function to create a test PDF with multiple pages
fn create_test_pdf(path: &str, page_count: usize) -> Result<()> {
    let mut doc = Document::new();
    
    for i in 1..=page_count {
        let mut page = Page::a4();
        page.text()
            .set_font(Font::Helvetica, 24.0)
            .at(100.0, 700.0)
            .write(&format!("Test Page {}", i))?;
        doc.add_page(page);
    }
    
    doc.save(path)?;
    Ok(())
}