//! Semantic marking for AI-Ready PDFs
//!
//! This module provides functionality to mark PDF regions with semantic meaning,
//! making PDFs more accessible to AI/ML processing pipelines.

mod entity;
mod export;
mod marking;

pub use entity::{Entity, EntityMetadata, EntityType};
pub use export::{EntityMap, ExportFormat};
pub use marking::{EntityBuilder, SemanticMarker};

/// Trait for types that support semantic marking
pub trait SemanticMarking {
    /// Mark a region with semantic meaning
    fn mark_region(&mut self, bounds: crate::graphics::Rectangle) -> EntityBuilder;

    /// Add a schema definition to the document
    fn add_schema(&mut self, schema_url: &str);

    /// Export all marked entities
    fn export_entities(&self) -> EntityMap;
}

#[cfg(feature = "pro")]
pub mod pro {
    //! PRO edition features for semantic marking
    use super::*;

    /// Extended entity types for PRO edition
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ProEntityType {
        Invoice,
        Receipt,
        Contract,
        Resume,
        MedicalRecord,
        Custom(u32),
    }
}

#[cfg(test)]
mod tests;
