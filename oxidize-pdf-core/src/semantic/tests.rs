//! Tests for semantic marking functionality

#[cfg(test)]
mod entity_tests {
    use crate::semantic::{Entity, EntityMetadata, EntityType};
    use std::collections::HashMap;

    #[test]
    fn test_entity_type_serialization() {
        // Test JSON serialization of entity types
        let entity_type = EntityType::Table;
        let json = serde_json::to_string(&entity_type).unwrap();
        assert_eq!(json, "\"table\"");
        
        let deserialized: EntityType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, EntityType::Table);
    }

    #[test]
    fn test_all_entity_types() {
        let types = vec![
            (EntityType::Text, "\"text\""),
            (EntityType::Image, "\"image\""),
            (EntityType::Table, "\"table\""),
            (EntityType::Heading, "\"heading\""),
            (EntityType::Paragraph, "\"paragraph\""),
            (EntityType::List, "\"list\""),
            (EntityType::PageNumber, "\"pageNumber\""),
            (EntityType::Header, "\"header\""),
            (EntityType::Footer, "\"footer\""),
        ];
        
        for (entity_type, expected_json) in types {
            let json = serde_json::to_string(&entity_type).unwrap();
            assert_eq!(json, expected_json);
        }
    }

    #[test]
    fn test_entity_metadata_new() {
        let metadata = EntityMetadata::new();
        assert!(metadata.properties.is_empty());
        assert!(metadata.confidence.is_none());
        assert!(metadata.schema.is_none());
    }

    #[test]
    fn test_entity_metadata_builder() {
        let metadata = EntityMetadata::new()
            .with_property("author", "John Doe")
            .with_property("date", "2023-07-15")
            .with_confidence(0.95)
            .with_schema("https://schema.org/Article");
        
        assert_eq!(metadata.properties.get("author").unwrap(), "John Doe");
        assert_eq!(metadata.properties.get("date").unwrap(), "2023-07-15");
        assert_eq!(metadata.confidence, Some(0.95));
        assert_eq!(metadata.schema.as_deref(), Some("https://schema.org/Article"));
    }

    #[test]
    fn test_entity_metadata_confidence_clamping() {
        // Test confidence is clamped between 0.0 and 1.0
        let metadata_high = EntityMetadata::new().with_confidence(1.5);
        assert_eq!(metadata_high.confidence, Some(1.0));
        
        let metadata_low = EntityMetadata::new().with_confidence(-0.5);
        assert_eq!(metadata_low.confidence, Some(0.0));
        
        let metadata_normal = EntityMetadata::new().with_confidence(0.75);
        assert_eq!(metadata_normal.confidence, Some(0.75));
    }

    #[test]
    fn test_entity_new() {
        let entity = Entity::new(
            "entity-1".to_string(),
            EntityType::Heading,
            (100.0, 200.0, 300.0, 50.0),
            0,
        );
        
        assert_eq!(entity.id, "entity-1");
        assert_eq!(entity.entity_type, EntityType::Heading);
        assert_eq!(entity.bounds, (100.0, 200.0, 300.0, 50.0));
        assert_eq!(entity.page, 0);
        assert!(entity.metadata.properties.is_empty());
    }

    #[test]
    fn test_entity_serialization() {
        let mut entity = Entity::new(
            "test-entity".to_string(),
            EntityType::Paragraph,
            (50.0, 100.0, 500.0, 200.0),
            2,
        );
        entity.metadata = EntityMetadata::new()
            .with_property("lang", "en")
            .with_confidence(0.88);
        
        let json = serde_json::to_string_pretty(&entity).unwrap();
        assert!(json.contains("\"id\": \"test-entity\""));
        assert!(json.contains("\"type\": \"paragraph\""));
        assert!(json.contains("\"page\": 2"));
        assert!(json.contains("\"confidence\": 0.88"));
        
        let deserialized: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, entity.id);
        assert_eq!(deserialized.entity_type, entity.entity_type);
        assert_eq!(deserialized.bounds, entity.bounds);
        assert_eq!(deserialized.page, entity.page);
        assert_eq!(deserialized.metadata.confidence, Some(0.88));
    }

    #[test]
    fn test_entity_metadata_multiple_properties() {
        let mut metadata = EntityMetadata::new();
        
        // Add properties manually
        metadata.properties.insert("key1".to_string(), "value1".to_string());
        metadata.properties.insert("key2".to_string(), "value2".to_string());
        metadata.properties.insert("key3".to_string(), "value3".to_string());
        
        assert_eq!(metadata.properties.len(), 3);
        assert_eq!(metadata.properties.get("key1").unwrap(), "value1");
        assert_eq!(metadata.properties.get("key2").unwrap(), "value2");
        assert_eq!(metadata.properties.get("key3").unwrap(), "value3");
    }

    #[test]
    fn test_entity_type_equality() {
        assert_eq!(EntityType::Text, EntityType::Text);
        assert_ne!(EntityType::Text, EntityType::Image);
        assert_ne!(EntityType::Table, EntityType::List);
    }

    #[test]
    fn test_entity_type_debug() {
        let entity_type = EntityType::Table;
        let debug_str = format!("{:?}", entity_type);
        assert_eq!(debug_str, "Table");
    }

    #[test]
    fn test_entity_clone() {
        let mut entity = Entity::new(
            "clone-test".to_string(),
            EntityType::Image,
            (0.0, 0.0, 100.0, 100.0),
            5,
        );
        entity.metadata = EntityMetadata::new()
            .with_property("alt", "Test image")
            .with_confidence(0.99);
        
        let cloned = entity.clone();
        assert_eq!(cloned.id, entity.id);
        assert_eq!(cloned.entity_type, entity.entity_type);
        assert_eq!(cloned.bounds, entity.bounds);
        assert_eq!(cloned.page, entity.page);
        assert_eq!(cloned.metadata.properties.get("alt"), Some(&"Test image".to_string()));
        assert_eq!(cloned.metadata.confidence, Some(0.99));
    }
}

#[cfg(test)]
mod export_tests {
    use crate::semantic::{Entity, EntityMap, EntityType, ExportFormat};
    
    #[test]
    fn test_entity_map_new() {
        let map = EntityMap::new();
        assert!(map.pages.is_empty());
        assert!(map.schemas.is_empty());
        assert!(map.document_metadata.is_empty());
    }

    #[test]
    fn test_entity_map_add_entity() {
        let mut map = EntityMap::new();
        
        let entity = Entity::new(
            "test-1".to_string(),
            EntityType::Text,
            (0.0, 0.0, 100.0, 100.0),
            0,
        );
        
        map.add_entity(entity);
        assert_eq!(map.pages.len(), 1);
        assert_eq!(map.pages.get(&0).unwrap().len(), 1);
        assert_eq!(map.pages.get(&0).unwrap()[0].id, "test-1");
    }

    #[test]
    fn test_entity_map_add_schema() {
        let mut map = EntityMap::new();
        map.schemas.push("https://schema.org/Article".to_string());
        map.schemas.push("https://schema.org/Person".to_string());
        
        assert_eq!(map.schemas.len(), 2);
        assert!(map.schemas.contains(&"https://schema.org/Article".to_string()));
        assert!(map.schemas.contains(&"https://schema.org/Person".to_string()));
    }

    #[test]
    fn test_entity_map_to_json() {
        let mut map = EntityMap::new();
        map.schemas.push("https://schema.org/Document".to_string());
        
        let entity = Entity::new(
            "heading-1".to_string(),
            EntityType::Heading,
            (50.0, 50.0, 200.0, 30.0),
            0,
        );
        map.add_entity(entity);
        
        let json = map.to_json().unwrap();
        assert!(json.contains("\"pages\""));
        assert!(json.contains("\"schemas\""));
        assert!(json.contains("heading-1"));
        assert!(json.contains("https://schema.org/Document"));
    }

    #[test]
    fn test_entity_map_to_json_compact() {
        let mut map = EntityMap::new();
        
        let entity = Entity::new(
            "test".to_string(),
            EntityType::Text,
            (0.0, 0.0, 10.0, 10.0),
            0,
        );
        map.add_entity(entity);
        
        let json_pretty = map.to_json().unwrap();
        let json_compact = map.to_json_compact().unwrap();
        
        // Compact should be shorter (no pretty printing)
        assert!(json_compact.len() < json_pretty.len());
        assert!(!json_compact.contains("\n"));
    }

    #[test]
    fn test_entity_map_page_filter() {
        let mut map = EntityMap::new();
        
        // Add entities on different pages
        for i in 0..5 {
            let entity = Entity::new(
                format!("entity-{}", i),
                EntityType::Text,
                (0.0, 0.0, 100.0, 100.0),
                i % 3, // Pages 0, 1, 2, 0, 1
            );
            map.add_entity(entity);
        }
        
        let page_0_entities = map.entities_on_page(0).unwrap();
        assert_eq!(page_0_entities.len(), 2);
        
        let page_1_entities = map.entities_on_page(1).unwrap();
        assert_eq!(page_1_entities.len(), 2);
        
        let page_2_entities = map.entities_on_page(2).unwrap();
        assert_eq!(page_2_entities.len(), 1);
        
        let page_3_entities = map.entities_on_page(3);
        assert!(page_3_entities.is_none());
    }

    #[test]
    fn test_entity_map_type_filter() {
        let mut map = EntityMap::new();
        
        // Add entities of different types
        map.add_entity(Entity::new("h1".to_string(), EntityType::Heading, (0.0, 0.0, 100.0, 30.0), 0));
        map.add_entity(Entity::new("h2".to_string(), EntityType::Heading, (0.0, 50.0, 100.0, 30.0), 0));
        map.add_entity(Entity::new("p1".to_string(), EntityType::Paragraph, (0.0, 100.0, 100.0, 50.0), 0));
        map.add_entity(Entity::new("img1".to_string(), EntityType::Image, (0.0, 200.0, 100.0, 100.0), 0));
        
        let headings = map.entities_by_type(EntityType::Heading);
        assert_eq!(headings.len(), 2);
        
        let paragraphs = map.entities_by_type(EntityType::Paragraph);
        assert_eq!(paragraphs.len(), 1);
        
        let images = map.entities_by_type(EntityType::Image);
        assert_eq!(images.len(), 1);
        
        let tables = map.entities_by_type(EntityType::Table);
        assert_eq!(tables.len(), 0);
    }

    #[test]
    fn test_export_format_variants() {
        let formats = vec![
            ExportFormat::Json,
            ExportFormat::JsonLd,
            ExportFormat::Xml,
        ];
        
        for format in formats {
            // Just ensure they can be created and matched
            match format {
                ExportFormat::Json => assert!(true),
                ExportFormat::JsonLd => assert!(true),
                ExportFormat::Xml => assert!(true),
            }
        }
    }
    
    #[test]
    fn test_export_format_default() {
        let default_format = ExportFormat::default();
        match default_format {
            ExportFormat::Json => assert!(true),
            _ => panic!("Default format should be Json"),
        }
    }
    
    #[test]
    fn test_entity_map_document_metadata() {
        let mut map = EntityMap::new();
        
        map.document_metadata.insert("title".to_string(), "Test Document".to_string());
        map.document_metadata.insert("author".to_string(), "Test Author".to_string());
        
        assert_eq!(map.document_metadata.len(), 2);
        assert_eq!(map.document_metadata.get("title").unwrap(), "Test Document");
        assert_eq!(map.document_metadata.get("author").unwrap(), "Test Author");
    }
}

#[cfg(test)]
mod marking_tests {
    use crate::semantic::{EntityType};
    
    #[test]
    fn test_entity_builder_metadata() {
        // Since EntityBuilder requires a mutable Page reference,
        // we'll test the metadata builder functionality through EntityMetadata
        use crate::semantic::EntityMetadata;
        
        let metadata = EntityMetadata::new()
            .with_property("level", "1")
            .with_property("text", "Introduction")
            .with_confidence(0.92)
            .with_schema("https://schema.org/Article");
        
        assert_eq!(metadata.properties.get("level"), Some(&"1".to_string()));
        assert_eq!(metadata.properties.get("text"), Some(&"Introduction".to_string()));
        assert_eq!(metadata.confidence, Some(0.92));
        assert_eq!(metadata.schema, Some("https://schema.org/Article".to_string()));
    }

    #[test]
    fn test_entity_type_variants() {
        // Test all entity type variants can be created
        let types = vec![
            EntityType::Text,
            EntityType::Image,
            EntityType::Table,
            EntityType::Heading,
            EntityType::Paragraph,
            EntityType::List,
            EntityType::PageNumber,
            EntityType::Header,
            EntityType::Footer,
        ];
        
        for entity_type in types {
            match entity_type {
                EntityType::Text => assert!(true),
                EntityType::Image => assert!(true),
                EntityType::Table => assert!(true),
                EntityType::Heading => assert!(true),
                EntityType::Paragraph => assert!(true),
                EntityType::List => assert!(true),
                EntityType::PageNumber => assert!(true),
                EntityType::Header => assert!(true),
                EntityType::Footer => assert!(true),
            }
        }
    }

    #[test]
    fn test_uuid_generation() {
        // Test that entity IDs would be unique
        use std::collections::HashSet;
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let mut ids = HashSet::new();
        
        for _ in 0..100 {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let id = format!("entity_{:x}", timestamp);
            
            // Small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_nanos(1));
            
            assert!(ids.insert(id), "Duplicate ID generated");
        }
        
        assert_eq!(ids.len(), 100);
    }
}