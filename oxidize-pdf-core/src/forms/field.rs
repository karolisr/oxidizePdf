//! Form field and widget definitions

use crate::geometry::Rectangle;
use crate::graphics::Color;
use crate::objects::{Dictionary, Object};

/// Field flags according to ISO 32000-1 Table 221
#[derive(Debug, Clone, Copy, Default)]
pub struct FieldFlags {
    /// Field is read-only
    pub read_only: bool,
    /// Field is required
    pub required: bool,
    /// Field should not be exported
    pub no_export: bool,
}

impl FieldFlags {
    /// Convert to PDF flags integer
    pub fn to_flags(&self) -> u32 {
        let mut flags = 0u32;
        if self.read_only {
            flags |= 1 << 0;
        }
        if self.required {
            flags |= 1 << 1;
        }
        if self.no_export {
            flags |= 1 << 2;
        }
        flags
    }
}

/// Field options
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct FieldOptions {
    /// Field flags
    pub flags: FieldFlags,
    /// Default appearance string
    pub default_appearance: Option<String>,
    /// Quadding (justification): 0=left, 1=center, 2=right
    pub quadding: Option<i32>,
}


/// Widget appearance settings
#[derive(Debug, Clone)]
pub struct WidgetAppearance {
    /// Border color
    pub border_color: Option<Color>,
    /// Background color
    pub background_color: Option<Color>,
    /// Border width
    pub border_width: f64,
    /// Border style: S (solid), D (dashed), B (beveled), I (inset), U (underline)
    pub border_style: BorderStyle,
}

/// Border style for widgets
#[derive(Debug, Clone, Copy)]
pub enum BorderStyle {
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

impl BorderStyle {
    /// Get PDF name
    pub fn pdf_name(&self) -> &'static str {
        match self {
            BorderStyle::Solid => "S",
            BorderStyle::Dashed => "D",
            BorderStyle::Beveled => "B",
            BorderStyle::Inset => "I",
            BorderStyle::Underline => "U",
        }
    }
}

impl Default for WidgetAppearance {
    fn default() -> Self {
        Self {
            border_color: Some(Color::black()),
            background_color: None,
            border_width: 1.0,
            border_style: BorderStyle::Solid,
        }
    }
}

/// Widget annotation for form field
#[derive(Debug, Clone)]
pub struct Widget {
    /// Rectangle for widget position
    pub rect: Rectangle,
    /// Appearance settings
    pub appearance: WidgetAppearance,
    /// Parent field reference (will be set by FormManager)
    pub parent: Option<String>,
}

impl Widget {
    /// Create a new widget
    pub fn new(rect: Rectangle) -> Self {
        Self {
            rect,
            appearance: WidgetAppearance::default(),
            parent: None,
        }
    }

    /// Set appearance
    pub fn with_appearance(mut self, appearance: WidgetAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    /// Convert to annotation dictionary
    pub fn to_annotation_dict(&self) -> Dictionary {
        let mut dict = Dictionary::new();

        // Annotation type
        dict.set("Type", Object::Name("Annot".to_string()));
        dict.set("Subtype", Object::Name("Widget".to_string()));

        // Rectangle
        let rect_array = vec![
            Object::Real(self.rect.lower_left.x),
            Object::Real(self.rect.lower_left.y),
            Object::Real(self.rect.upper_right.x),
            Object::Real(self.rect.upper_right.y),
        ];
        dict.set("Rect", Object::Array(rect_array));

        // Border style
        let mut bs_dict = Dictionary::new();
        bs_dict.set("W", Object::Real(self.appearance.border_width));
        bs_dict.set(
            "S",
            Object::Name(self.appearance.border_style.pdf_name().to_string()),
        );
        dict.set("BS", Object::Dictionary(bs_dict));

        // Appearance characteristics
        let mut mk_dict = Dictionary::new();

        if let Some(border_color) = &self.appearance.border_color {
            let bc = match border_color {
                Color::Rgb(r, g, b) => vec![Object::Real(*r), Object::Real(*g), Object::Real(*b)],
                Color::Gray(g) => vec![Object::Real(*g)],
                Color::Cmyk(c, m, y, k) => vec![
                    Object::Real(*c),
                    Object::Real(*m),
                    Object::Real(*y),
                    Object::Real(*k),
                ],
            };
            mk_dict.set("BC", Object::Array(bc));
        }

        if let Some(bg_color) = &self.appearance.background_color {
            let bg = match bg_color {
                Color::Rgb(r, g, b) => vec![Object::Real(*r), Object::Real(*g), Object::Real(*b)],
                Color::Gray(g) => vec![Object::Real(*g)],
                Color::Cmyk(c, m, y, k) => vec![
                    Object::Real(*c),
                    Object::Real(*m),
                    Object::Real(*y),
                    Object::Real(*k),
                ],
            };
            mk_dict.set("BG", Object::Array(bg));
        }

        dict.set("MK", Object::Dictionary(mk_dict));

        // Flags - print flag
        dict.set("F", Object::Integer(4));

        dict
    }
}

/// Base field trait
pub trait Field {
    /// Get field name
    fn name(&self) -> &str;

    /// Get field type
    fn field_type(&self) -> &'static str;

    /// Convert to dictionary
    fn to_dict(&self) -> Dictionary;
}

/// Form field with widget
#[derive(Debug, Clone)]
pub struct FormField {
    /// Field dictionary
    pub field_dict: Dictionary,
    /// Associated widgets
    pub widgets: Vec<Widget>,
}

impl FormField {
    /// Create from field dictionary
    pub fn new(field_dict: Dictionary) -> Self {
        Self {
            field_dict,
            widgets: Vec::new(),
        }
    }

    /// Add a widget
    pub fn add_widget(&mut self, widget: Widget) {
        self.widgets.push(widget);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_field_flags() {
        let flags = FieldFlags {
            read_only: true,
            required: true,
            no_export: false,
        };

        assert_eq!(flags.to_flags(), 3); // bits 0 and 1 set
    }

    #[test]
    fn test_border_style() {
        assert_eq!(BorderStyle::Solid.pdf_name(), "S");
        assert_eq!(BorderStyle::Dashed.pdf_name(), "D");
        assert_eq!(BorderStyle::Beveled.pdf_name(), "B");
        assert_eq!(BorderStyle::Inset.pdf_name(), "I");
        assert_eq!(BorderStyle::Underline.pdf_name(), "U");
    }

    #[test]
    fn test_widget_creation() {
        let rect = Rectangle::new(Point::new(100.0, 100.0), Point::new(200.0, 120.0));

        let widget = Widget::new(rect);
        assert_eq!(widget.rect.lower_left.x, 100.0);
        assert_eq!(widget.appearance.border_width, 1.0);
    }

    #[test]
    fn test_widget_annotation_dict() {
        let rect = Rectangle::new(Point::new(50.0, 50.0), Point::new(150.0, 70.0));

        let appearance = WidgetAppearance {
            border_color: Some(Color::Rgb(0.0, 0.0, 1.0)),
            background_color: Some(Color::Gray(0.9)),
            border_width: 2.0,
            border_style: BorderStyle::Solid,
        };

        let widget = Widget::new(rect).with_appearance(appearance);
        let dict = widget.to_annotation_dict();

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(
            dict.get("Subtype"),
            Some(&Object::Name("Widget".to_string()))
        );
        assert!(dict.get("Rect").is_some());
        assert!(dict.get("BS").is_some());
        assert!(dict.get("MK").is_some());
    }

    #[test]
    fn test_field_options_default() {
        let options = FieldOptions::default();
        assert!(!options.flags.read_only);
        assert!(!options.flags.required);
        assert!(!options.flags.no_export);
        assert!(options.default_appearance.is_none());
        assert!(options.quadding.is_none());
    }
}
