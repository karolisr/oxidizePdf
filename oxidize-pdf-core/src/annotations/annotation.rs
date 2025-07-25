//! Base annotation types and management

use crate::geometry::Rectangle;
use crate::graphics::Color;
use crate::objects::{Dictionary, Object, ObjectReference};
use std::collections::HashMap;

/// Annotation types according to ISO 32000-1 Table 169
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnnotationType {
    /// Text annotation (sticky note)
    Text,
    /// Link annotation
    Link,
    /// Free text annotation
    FreeText,
    /// Line annotation
    Line,
    /// Square annotation
    Square,
    /// Circle annotation
    Circle,
    /// Polygon annotation
    Polygon,
    /// Polyline annotation
    PolyLine,
    /// Highlight annotation
    Highlight,
    /// Underline annotation
    Underline,
    /// Squiggly underline annotation
    Squiggly,
    /// Strikeout annotation
    StrikeOut,
    /// Rubber stamp annotation
    Stamp,
    /// Caret annotation
    Caret,
    /// Ink annotation
    Ink,
    /// Popup annotation
    Popup,
    /// File attachment annotation
    FileAttachment,
    /// Sound annotation
    Sound,
    /// Movie annotation
    Movie,
    /// Widget annotation (form field)
    Widget,
    /// Screen annotation
    Screen,
    /// Printer mark annotation
    PrinterMark,
    /// Trap network annotation
    TrapNet,
    /// Watermark annotation
    Watermark,
}

impl AnnotationType {
    /// Get PDF subtype name
    pub fn pdf_name(&self) -> &'static str {
        match self {
            AnnotationType::Text => "Text",
            AnnotationType::Link => "Link",
            AnnotationType::FreeText => "FreeText",
            AnnotationType::Line => "Line",
            AnnotationType::Square => "Square",
            AnnotationType::Circle => "Circle",
            AnnotationType::Polygon => "Polygon",
            AnnotationType::PolyLine => "PolyLine",
            AnnotationType::Highlight => "Highlight",
            AnnotationType::Underline => "Underline",
            AnnotationType::Squiggly => "Squiggly",
            AnnotationType::StrikeOut => "StrikeOut",
            AnnotationType::Stamp => "Stamp",
            AnnotationType::Caret => "Caret",
            AnnotationType::Ink => "Ink",
            AnnotationType::Popup => "Popup",
            AnnotationType::FileAttachment => "FileAttachment",
            AnnotationType::Sound => "Sound",
            AnnotationType::Movie => "Movie",
            AnnotationType::Widget => "Widget",
            AnnotationType::Screen => "Screen",
            AnnotationType::PrinterMark => "PrinterMark",
            AnnotationType::TrapNet => "TrapNet",
            AnnotationType::Watermark => "Watermark",
        }
    }
}

/// Annotation flags according to ISO 32000-1 Section 12.5.3
#[derive(Debug, Clone, Copy, Default)]
pub struct AnnotationFlags {
    /// Annotation is invisible
    pub invisible: bool,
    /// Annotation is hidden
    pub hidden: bool,
    /// Annotation should be printed
    pub print: bool,
    /// Annotation should not zoom
    pub no_zoom: bool,
    /// Annotation should not rotate
    pub no_rotate: bool,
    /// Annotation should not be viewed
    pub no_view: bool,
    /// Annotation is read-only
    pub read_only: bool,
    /// Annotation is locked
    pub locked: bool,
    /// Annotation content is locked
    pub locked_contents: bool,
}

impl AnnotationFlags {
    /// Convert to PDF flags integer
    pub fn to_flags(&self) -> u32 {
        let mut flags = 0u32;
        if self.invisible {
            flags |= 1 << 0;
        }
        if self.hidden {
            flags |= 1 << 1;
        }
        if self.print {
            flags |= 1 << 2;
        }
        if self.no_zoom {
            flags |= 1 << 3;
        }
        if self.no_rotate {
            flags |= 1 << 4;
        }
        if self.no_view {
            flags |= 1 << 5;
        }
        if self.read_only {
            flags |= 1 << 6;
        }
        if self.locked {
            flags |= 1 << 7;
        }
        if self.locked_contents {
            flags |= 1 << 9;
        }
        flags
    }
}

/// Border style for annotations
#[derive(Debug, Clone)]
pub struct BorderStyle {
    /// Width in points
    pub width: f64,
    /// Style: S (solid), D (dashed), B (beveled), I (inset), U (underline)
    pub style: BorderStyleType,
    /// Dash pattern for dashed borders
    pub dash_pattern: Option<Vec<f64>>,
}

/// Border style type
#[derive(Debug, Clone, Copy)]
pub enum BorderStyleType {
    /// Solid border
    Solid,
    /// Dashed border
    Dashed,
    /// Beveled border
    Beveled,
    /// Inset border
    Inset,
    /// Underline only
    Underline,
}

impl BorderStyleType {
    /// Get PDF name
    pub fn pdf_name(&self) -> &'static str {
        match self {
            BorderStyleType::Solid => "S",
            BorderStyleType::Dashed => "D",
            BorderStyleType::Beveled => "B",
            BorderStyleType::Inset => "I",
            BorderStyleType::Underline => "U",
        }
    }
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            style: BorderStyleType::Solid,
            dash_pattern: None,
        }
    }
}

/// Base annotation structure
#[derive(Debug, Clone)]
pub struct Annotation {
    /// Annotation type
    pub annotation_type: AnnotationType,
    /// Rectangle defining annotation position
    pub rect: Rectangle,
    /// Optional content text
    pub contents: Option<String>,
    /// Optional annotation name
    pub name: Option<String>,
    /// Modification date
    pub modified: Option<String>,
    /// Flags
    pub flags: AnnotationFlags,
    /// Border style
    pub border: Option<BorderStyle>,
    /// Color
    pub color: Option<Color>,
    /// Page reference (set by manager)
    pub page: Option<ObjectReference>,
    /// Additional properties specific to annotation type
    pub properties: Dictionary,
}

impl Annotation {
    /// Create a new annotation
    pub fn new(annotation_type: AnnotationType, rect: Rectangle) -> Self {
        Self {
            annotation_type,
            rect,
            contents: None,
            name: None,
            modified: None,
            flags: AnnotationFlags {
                print: true,
                ..Default::default()
            },
            border: None,
            color: None,
            page: None,
            properties: Dictionary::new(),
        }
    }

    /// Set contents
    pub fn with_contents(mut self, contents: impl Into<String>) -> Self {
        self.contents = Some(contents.into());
        self
    }

    /// Set name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set border
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = Some(border);
        self
    }

    /// Set flags
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Convert to PDF dictionary
    pub fn to_dict(&self) -> Dictionary {
        let mut dict = Dictionary::new();

        // Required fields
        dict.set("Type", Object::Name("Annot".to_string()));
        dict.set(
            "Subtype",
            Object::Name(self.annotation_type.pdf_name().to_string()),
        );

        // Rectangle
        let rect_array = vec![
            Object::Real(self.rect.lower_left.x),
            Object::Real(self.rect.lower_left.y),
            Object::Real(self.rect.upper_right.x),
            Object::Real(self.rect.upper_right.y),
        ];
        dict.set("Rect", Object::Array(rect_array));

        // Optional fields
        if let Some(ref contents) = self.contents {
            dict.set("Contents", Object::String(contents.clone()));
        }

        if let Some(ref name) = self.name {
            dict.set("NM", Object::String(name.clone()));
        }

        if let Some(ref modified) = self.modified {
            dict.set("M", Object::String(modified.clone()));
        }

        // Flags
        let flags = self.flags.to_flags();
        if flags != 0 {
            dict.set("F", Object::Integer(flags as i64));
        }

        // Border
        if let Some(ref border) = self.border {
            let mut bs_dict = Dictionary::new();
            bs_dict.set("W", Object::Real(border.width));
            bs_dict.set("S", Object::Name(border.style.pdf_name().to_string()));

            if let Some(ref dash) = border.dash_pattern {
                let dash_array: Vec<Object> = dash.iter().map(|&d| Object::Real(d)).collect();
                bs_dict.set("D", Object::Array(dash_array));
            }

            dict.set("BS", Object::Dictionary(bs_dict));
        }

        // Color
        if let Some(ref color) = self.color {
            let c = match color {
                Color::Rgb(r, g, b) => vec![Object::Real(*r), Object::Real(*g), Object::Real(*b)],
                Color::Gray(g) => vec![Object::Real(*g)],
                Color::Cmyk(c, m, y, k) => vec![
                    Object::Real(*c),
                    Object::Real(*m),
                    Object::Real(*y),
                    Object::Real(*k),
                ],
            };
            dict.set("C", Object::Array(c));
        }

        // Page reference
        if let Some(page) = self.page {
            dict.set("P", Object::Reference(page));
        }

        // Merge additional properties
        for (key, value) in self.properties.iter() {
            dict.set(key, value.clone());
        }

        dict
    }
}

/// Annotation manager
#[derive(Debug)]
pub struct AnnotationManager {
    /// Annotations by page
    annotations: HashMap<ObjectReference, Vec<Annotation>>,
    /// Next annotation ID
    next_id: u32,
}

impl AnnotationManager {
    /// Create a new annotation manager
    pub fn new() -> Self {
        Self {
            annotations: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add an annotation to a page
    pub fn add_annotation(
        &mut self,
        page_ref: ObjectReference,
        mut annotation: Annotation,
    ) -> ObjectReference {
        annotation.page = Some(page_ref);

        let annot_ref = ObjectReference::new(self.next_id, 0);
        self.next_id += 1;

        self.annotations
            .entry(page_ref)
            .or_default()
            .push(annotation);

        annot_ref
    }

    /// Get annotations for a page
    pub fn get_page_annotations(&self, page_ref: &ObjectReference) -> Option<&Vec<Annotation>> {
        self.annotations.get(page_ref)
    }

    /// Get all annotations
    pub fn all_annotations(&self) -> &HashMap<ObjectReference, Vec<Annotation>> {
        &self.annotations
    }
}

impl Default for AnnotationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_annotation_type() {
        assert_eq!(AnnotationType::Text.pdf_name(), "Text");
        assert_eq!(AnnotationType::Link.pdf_name(), "Link");
        assert_eq!(AnnotationType::Highlight.pdf_name(), "Highlight");
    }

    #[test]
    fn test_annotation_flags() {
        let flags = AnnotationFlags {
            print: true,
            read_only: true,
            ..Default::default()
        };

        assert_eq!(flags.to_flags(), 68); // bits 2 and 6 set
    }

    #[test]
    fn test_border_style() {
        let border = BorderStyle {
            width: 2.0,
            style: BorderStyleType::Dashed,
            dash_pattern: Some(vec![3.0, 1.0]),
        };

        assert_eq!(border.width, 2.0);
        assert_eq!(border.style.pdf_name(), "D");
    }

    #[test]
    fn test_annotation_creation() {
        let rect = Rectangle::new(Point::new(100.0, 100.0), Point::new(200.0, 200.0));

        let annotation = Annotation::new(AnnotationType::Text, rect)
            .with_contents("Test annotation")
            .with_color(Color::Rgb(1.0, 0.0, 0.0));

        assert_eq!(annotation.annotation_type, AnnotationType::Text);
        assert_eq!(annotation.contents, Some("Test annotation".to_string()));
        assert!(annotation.color.is_some());
    }

    #[test]
    fn test_annotation_to_dict() {
        let rect = Rectangle::new(Point::new(50.0, 50.0), Point::new(150.0, 150.0));

        let annotation =
            Annotation::new(AnnotationType::Square, rect).with_contents("Square annotation");

        let dict = annotation.to_dict();
        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(
            dict.get("Subtype"),
            Some(&Object::Name("Square".to_string()))
        );
        assert!(dict.get("Rect").is_some());
        assert_eq!(
            dict.get("Contents"),
            Some(&Object::String("Square annotation".to_string()))
        );
    }

    #[test]
    fn test_annotation_manager() {
        let mut manager = AnnotationManager::new();
        let page_ref = ObjectReference::new(1, 0);

        let rect = Rectangle::new(Point::new(100.0, 100.0), Point::new(200.0, 200.0));

        let annotation = Annotation::new(AnnotationType::Text, rect);
        let annot_ref = manager.add_annotation(page_ref, annotation);

        assert_eq!(annot_ref.number(), 1);
        assert!(manager.get_page_annotations(&page_ref).is_some());
        assert_eq!(manager.get_page_annotations(&page_ref).unwrap().len(), 1);
    }
}
