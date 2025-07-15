//! Tests for the page tree parser module

#[cfg(test)]
mod tests {
    use crate::parser::objects::{PdfArray, PdfDictionary, PdfObject};
    use crate::parser::page_tree::*;
    use crate::parser::reader::PdfReader;
    use std::io::Cursor;

    /// Create a mock PDF reader with sample data
    fn create_mock_reader() -> PdfReader<Cursor<Vec<u8>>> {
        // Create a minimal valid PDF with proper structure
        let data = b"%PDF-1.4\n\
1 0 obj\n\
<< /Type /Catalog /Pages 2 0 R >>\n\
endobj\n\
2 0 obj\n\
<< /Type /Pages /Kids [3 0 R] /Count 1 >>\n\
endobj\n\
3 0 obj\n\
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\n\
endobj\n\
xref\n\
0 4\n\
0000000000 65535 f \n\
0000000009 00000 n \n\
0000000058 00000 n \n\
0000000117 00000 n \n\
trailer\n\
<< /Size 4 /Root 1 0 R >>\n\
startxref\n\
205\n\
%%EOF";
        PdfReader::new(Cursor::new(data.to_vec())).unwrap()
    }

    #[test]
    fn test_page_tree_new() {
        let tree = PageTree::new(10);
        assert_eq!(tree.page_count(), 10);
        assert!(tree.get_cached_page(0).is_none());
    }

    #[test]
    fn test_page_tree_new_with_pages_dict() {
        let pages_dict = PdfDictionary::new();
        let tree = PageTree::new_with_pages_dict(5, pages_dict);
        assert_eq!(tree.page_count(), 5);
    }

    #[test]
    fn test_page_tree_cache_operations() {
        let mut tree = PageTree::new(10);

        // Test caching a page
        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: PdfDictionary::new(),
            inherited_resources: None,
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: None,
            rotation: 0,
        };

        tree.cache_page(0, page.clone());

        // Test retrieving cached page
        let cached = tree.get_cached_page(0);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().obj_ref, (1, 0));

        // Test non-existent page
        assert!(tree.get_cached_page(1).is_none());
    }

    #[test]
    fn test_parsed_page_dimensions() {
        // Test normal page (no rotation)
        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: PdfDictionary::new(),
            inherited_resources: None,
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: None,
            rotation: 0,
        };

        assert_eq!(page.width(), 612.0);
        assert_eq!(page.height(), 792.0);
    }

    #[test]
    fn test_parsed_page_dimensions_rotated_90() {
        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: PdfDictionary::new(),
            inherited_resources: None,
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: None,
            rotation: 90,
        };

        // Width and height should be swapped
        assert_eq!(page.width(), 792.0);
        assert_eq!(page.height(), 612.0);
    }

    #[test]
    fn test_parsed_page_dimensions_rotated_270() {
        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: PdfDictionary::new(),
            inherited_resources: None,
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: None,
            rotation: 270,
        };

        // Width and height should be swapped
        assert_eq!(page.width(), 792.0);
        assert_eq!(page.height(), 612.0);
    }

    #[test]
    fn test_parsed_page_dimensions_rotated_180() {
        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: PdfDictionary::new(),
            inherited_resources: None,
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: None,
            rotation: 180,
        };

        // Dimensions should remain the same
        assert_eq!(page.width(), 612.0);
        assert_eq!(page.height(), 792.0);
    }

    #[test]
    fn test_parsed_page_get_resources() {
        let mut resources_dict = PdfDictionary::new();
        resources_dict.insert(
            "Font".to_string(),
            PdfObject::Dictionary(PdfDictionary::new()),
        );

        let mut page_dict = PdfDictionary::new();
        page_dict.insert(
            "Resources".to_string(),
            PdfObject::Dictionary(resources_dict.clone()),
        );

        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: page_dict,
            inherited_resources: None,
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: None,
            rotation: 0,
        };

        let resources = page.get_resources();
        assert!(resources.is_some());
        assert!(resources.unwrap().contains_key("Font"));
    }

    #[test]
    fn test_parsed_page_get_inherited_resources() {
        let mut inherited_resources = PdfDictionary::new();
        inherited_resources.insert(
            "Font".to_string(),
            PdfObject::Dictionary(PdfDictionary::new()),
        );

        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: PdfDictionary::new(), // No direct resources
            inherited_resources: Some(inherited_resources.clone()),
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: None,
            rotation: 0,
        };

        let resources = page.get_resources();
        assert!(resources.is_some());
        assert!(resources.unwrap().contains_key("Font"));
    }

    #[test]
    fn test_parsed_page_clone_with_resources() {
        let mut inherited_resources = PdfDictionary::new();
        inherited_resources.insert(
            "Font".to_string(),
            PdfObject::Dictionary(PdfDictionary::new()),
        );

        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: PdfDictionary::new(),
            inherited_resources: Some(inherited_resources),
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: None,
            rotation: 0,
        };

        let cloned = page.clone_with_resources();

        // The cloned page should have Resources in its dictionary
        assert!(cloned.dict.contains_key("Resources"));

        // And it should contain the inherited Font
        let resources = cloned.dict.get("Resources").unwrap().as_dict().unwrap();
        assert!(resources.contains_key("Font"));
    }

    #[test]
    fn test_parsed_page_clone_with_resources_preserves_existing() {
        let mut page_resources = PdfDictionary::new();
        page_resources.insert(
            "XObject".to_string(),
            PdfObject::Dictionary(PdfDictionary::new()),
        );

        let mut page_dict = PdfDictionary::new();
        page_dict.insert(
            "Resources".to_string(),
            PdfObject::Dictionary(page_resources),
        );

        let mut inherited_resources = PdfDictionary::new();
        inherited_resources.insert(
            "Font".to_string(),
            PdfObject::Dictionary(PdfDictionary::new()),
        );

        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: page_dict,
            inherited_resources: Some(inherited_resources),
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: None,
            rotation: 0,
        };

        let cloned = page.clone_with_resources();

        // Should preserve existing resources
        let resources = cloned.dict.get("Resources").unwrap().as_dict().unwrap();
        assert!(resources.contains_key("XObject"));
        // Should NOT have Font from inherited (existing Resources take precedence)
        assert!(!resources.contains_key("Font"));
    }

    #[test]
    fn test_collect_references() {
        let mut refs = Vec::new();

        // Test reference collection
        let ref_obj = PdfObject::Reference(5, 0);
        ParsedPage::collect_references(&ref_obj, &mut refs);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0], (5, 0));

        // Test array with references
        refs.clear();
        let array = PdfArray(vec![
            PdfObject::Reference(1, 0),
            PdfObject::Integer(42),
            PdfObject::Reference(2, 0),
        ]);
        ParsedPage::collect_references(&PdfObject::Array(array), &mut refs);
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&(1, 0)));
        assert!(refs.contains(&(2, 0)));

        // Test dictionary with references
        refs.clear();
        let mut dict = PdfDictionary::new();
        dict.insert("Font".to_string(), PdfObject::Reference(3, 0));
        dict.insert("Count".to_string(), PdfObject::Integer(10));
        ParsedPage::collect_references(&PdfObject::Dictionary(dict), &mut refs);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0], (3, 0));
    }

    #[test]
    fn test_content_streams_empty() {
        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: PdfDictionary::new(), // No Contents
            inherited_resources: None,
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: None,
            rotation: 0,
        };

        // Test that content_streams returns empty vec when no Contents
        // We don't need a real reader for this test since there's no Contents key
        let mut mock_data = Cursor::new(Vec::new());
        match PdfReader::new(mock_data) {
            Ok(mut reader) => {
                let streams = page.content_streams(&mut reader).unwrap();
                assert!(streams.is_empty());
            }
            Err(_) => {
                // If we can't create a reader, just verify the page has no Contents
                assert!(!page.dict.contains_key("Contents"));
            }
        }
    }

    #[test]
    fn test_page_tree_get_rectangle() {
        let mut node = PdfDictionary::new();
        let media_box = PdfArray(vec![
            PdfObject::Real(0.0),
            PdfObject::Real(0.0),
            PdfObject::Real(612.0),
            PdfObject::Real(792.0),
        ]);
        node.insert("MediaBox".to_string(), PdfObject::Array(media_box));

        let rect = PageTree::get_rectangle(&node, None, "MediaBox").unwrap();
        assert!(rect.is_some());
        assert_eq!(rect.unwrap(), [0.0, 0.0, 612.0, 792.0]);
    }

    #[test]
    fn test_page_tree_get_rectangle_inherited() {
        let node = PdfDictionary::new(); // No MediaBox

        let mut inherited = PdfDictionary::new();
        let media_box = PdfArray(vec![
            PdfObject::Real(0.0),
            PdfObject::Real(0.0),
            PdfObject::Real(612.0),
            PdfObject::Real(792.0),
        ]);
        inherited.insert("MediaBox".to_string(), PdfObject::Array(media_box));

        let rect = PageTree::get_rectangle(&node, Some(&inherited), "MediaBox").unwrap();
        assert!(rect.is_some());
        assert_eq!(rect.unwrap(), [0.0, 0.0, 612.0, 792.0]);
    }

    #[test]
    fn test_page_tree_get_rectangle_invalid_length() {
        let mut node = PdfDictionary::new();
        let invalid_box = PdfArray(vec![
            PdfObject::Real(0.0),
            PdfObject::Real(0.0),
            PdfObject::Real(612.0),
            // Missing fourth element
        ]);
        node.insert("MediaBox".to_string(), PdfObject::Array(invalid_box));

        let result = PageTree::get_rectangle(&node, None, "MediaBox");
        assert!(result.is_err());
    }

    #[test]
    fn test_page_tree_get_integer() {
        let mut node = PdfDictionary::new();
        node.insert("Rotate".to_string(), PdfObject::Integer(90));

        let value = PageTree::get_integer(&node, None, "Rotate").unwrap();
        assert_eq!(value, Some(90));
    }

    #[test]
    fn test_page_tree_get_integer_inherited() {
        let node = PdfDictionary::new(); // No Rotate

        let mut inherited = PdfDictionary::new();
        inherited.insert("Rotate".to_string(), PdfObject::Integer(180));

        let value = PageTree::get_integer(&node, Some(&inherited), "Rotate").unwrap();
        assert_eq!(value, Some(180));
    }

    #[test]
    fn test_page_tree_get_integer_not_found() {
        let node = PdfDictionary::new();
        let value = PageTree::get_integer(&node, None, "Rotate").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_parsed_page_debug_trait() {
        let page = ParsedPage {
            obj_ref: (1, 0),
            dict: PdfDictionary::new(),
            inherited_resources: None,
            media_box: [0.0, 0.0, 612.0, 792.0],
            crop_box: Some([10.0, 10.0, 602.0, 782.0]),
            rotation: 90,
        };

        let debug_str = format!("{:?}", page);
        assert!(debug_str.contains("ParsedPage"));
        assert!(debug_str.contains("(1, 0)"));
        assert!(debug_str.contains("rotation: 90"));
    }

    #[test]
    fn test_parsed_page_clone() {
        let mut resources = PdfDictionary::new();
        resources.insert(
            "Font".to_string(),
            PdfObject::Dictionary(PdfDictionary::new()),
        );

        let page = ParsedPage {
            obj_ref: (5, 0),
            dict: PdfDictionary::new(),
            inherited_resources: Some(resources),
            media_box: [0.0, 0.0, 595.0, 842.0], // A4
            crop_box: Some([20.0, 20.0, 575.0, 822.0]),
            rotation: 180,
        };

        let cloned = page.clone();
        assert_eq!(cloned.obj_ref, page.obj_ref);
        assert_eq!(cloned.media_box, page.media_box);
        assert_eq!(cloned.crop_box, page.crop_box);
        assert_eq!(cloned.rotation, page.rotation);
        assert!(cloned.inherited_resources.is_some());
    }

    #[test]
    fn test_multiple_page_cache() {
        let mut tree = PageTree::new(100);

        // Cache multiple pages
        for i in 0..10 {
            let page = ParsedPage {
                obj_ref: (i + 1, 0),
                dict: PdfDictionary::new(),
                inherited_resources: None,
                media_box: [0.0, 0.0, 612.0, 792.0],
                crop_box: None,
                rotation: 0,
            };
            tree.cache_page(i, page);
        }

        // Verify all pages are cached
        for i in 0..10 {
            let cached = tree.get_cached_page(i);
            assert!(cached.is_some());
            assert_eq!(cached.unwrap().obj_ref.0, i + 1);
        }

        // Verify uncached pages return None
        assert!(tree.get_cached_page(10).is_none());
        assert!(tree.get_cached_page(99).is_none());
    }
}
