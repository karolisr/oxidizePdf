//! Document outline (bookmarks) according to ISO 32000-1 Section 12.3.3

use crate::graphics::Color;
use crate::objects::{Array, Dictionary, Object, ObjectId};
use crate::structure::destination::Destination;
use std::collections::VecDeque;

/// Outline item flags
#[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub struct OutlineFlags {
    /// Italic text
    pub italic: bool,
    /// Bold text
    pub bold: bool,
}


impl OutlineFlags {
    /// Convert to integer flags
    pub fn to_int(&self) -> i64 {
        let mut flags = 0;
        if self.italic {
            flags |= 1;
        }
        if self.bold {
            flags |= 2;
        }
        flags
    }
}

/// Outline item (bookmark)
#[derive(Debug, Clone)]
pub struct OutlineItem {
    /// Item title
    pub title: String,
    /// Destination
    pub destination: Option<Destination>,
    /// Child items
    pub children: Vec<OutlineItem>,
    /// Text color
    pub color: Option<Color>,
    /// Text style flags
    pub flags: OutlineFlags,
    /// Whether item is open by default
    pub open: bool,
}

impl OutlineItem {
    /// Create new outline item
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            destination: None,
            children: Vec::new(),
            color: None,
            flags: OutlineFlags::default(),
            open: true,
        }
    }

    /// Set destination
    pub fn with_destination(mut self, dest: Destination) -> Self {
        self.destination = Some(dest);
        self
    }

    /// Add child item
    pub fn add_child(&mut self, child: OutlineItem) {
        self.children.push(child);
    }

    /// Set color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set bold
    pub fn bold(mut self) -> Self {
        self.flags.bold = true;
        self
    }

    /// Set italic
    pub fn italic(mut self) -> Self {
        self.flags.italic = true;
        self
    }

    /// Set closed by default
    pub fn closed(mut self) -> Self {
        self.open = false;
        self
    }

    /// Count total items in subtree
    pub fn count_all(&self) -> i64 {
        let mut count = 1; // Self
        for child in &self.children {
            count += child.count_all();
        }
        count
    }

    /// Count visible items (respecting open/closed state)
    pub fn count_visible(&self) -> i64 {
        let mut count = 1; // Self
        if self.open {
            for child in &self.children {
                count += child.count_visible();
            }
        }
        count
    }
}

/// Outline tree structure
pub struct OutlineTree {
    /// Root items
    pub items: Vec<OutlineItem>,
}

impl Default for OutlineTree {
    fn default() -> Self {
        Self::new()
    }
}

impl OutlineTree {
    /// Create new outline tree
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add root item
    pub fn add_item(&mut self, item: OutlineItem) {
        self.items.push(item);
    }

    /// Get total item count
    pub fn total_count(&self) -> i64 {
        self.items.iter().map(|item| item.count_all()).sum()
    }

    /// Get visible item count
    pub fn visible_count(&self) -> i64 {
        self.items.iter().map(|item| item.count_visible()).sum()
    }
}

/// Outline builder for creating outline hierarchy
pub struct OutlineBuilder {
    /// Current outline tree
    tree: OutlineTree,
    /// Stack for building hierarchy
    stack: VecDeque<OutlineItem>,
}

impl Default for OutlineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OutlineBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            tree: OutlineTree::new(),
            stack: VecDeque::new(),
        }
    }

    /// Add item at current level
    pub fn add_item(&mut self, item: OutlineItem) {
        if let Some(parent) = self.stack.back_mut() {
            parent.add_child(item);
        } else {
            self.tree.add_item(item);
        }
    }

    /// Push item and make it current parent
    pub fn push_item(&mut self, item: OutlineItem) {
        self.stack.push_back(item);
    }

    /// Pop current parent and add to tree
    pub fn pop_item(&mut self) {
        if let Some(item) = self.stack.pop_back() {
            if let Some(parent) = self.stack.back_mut() {
                parent.add_child(item);
            } else {
                self.tree.add_item(item);
            }
        }
    }

    /// Build the outline tree
    pub fn build(mut self) -> OutlineTree {
        // Pop any remaining items
        while !self.stack.is_empty() {
            self.pop_item();
        }
        self.tree
    }
}

/// Convert outline item to dictionary (for PDF generation)
#[allow(dead_code)]
pub fn outline_item_to_dict(
    item: &OutlineItem,
    parent_ref: ObjectId,
    first_ref: Option<ObjectId>,
    last_ref: Option<ObjectId>,
    prev_ref: Option<ObjectId>,
    next_ref: Option<ObjectId>,
) -> Dictionary {
    let mut dict = Dictionary::new();

    // Title
    dict.set("Title", Object::String(item.title.clone()));

    // Parent
    dict.set("Parent", Object::Reference(parent_ref));

    // Siblings
    if let Some(prev) = prev_ref {
        dict.set("Prev", Object::Reference(prev));
    }
    if let Some(next) = next_ref {
        dict.set("Next", Object::Reference(next));
    }

    // Children
    if !item.children.is_empty() {
        if let Some(first) = first_ref {
            dict.set("First", Object::Reference(first));
        }
        if let Some(last) = last_ref {
            dict.set("Last", Object::Reference(last));
        }

        // Count (negative if closed)
        let count = item.count_visible() - 1; // Exclude self
        dict.set(
            "Count",
            Object::Integer(if item.open { count } else { -count }),
        );
    }

    // Destination
    if let Some(dest) = &item.destination {
        dict.set("Dest", Object::Array(dest.to_array().into()));
    }

    // Color
    if let Some(color) = &item.color {
        let color_array = match color {
            Color::Rgb(r, g, b) => {
                Array::from(vec![Object::Real(*r), Object::Real(*g), Object::Real(*b)])
            }
            Color::Gray(g) => {
                Array::from(vec![Object::Real(*g), Object::Real(*g), Object::Real(*g)])
            }
            Color::Cmyk(c, m, y, k) => {
                // Convert CMYK to RGB approximation for outline color
                let r = (1.0 - c) * (1.0 - k);
                let g = (1.0 - m) * (1.0 - k);
                let b = (1.0 - y) * (1.0 - k);
                Array::from(vec![Object::Real(r), Object::Real(g), Object::Real(b)])
            }
        };
        dict.set("C", Object::Array(color_array.into()));
    }

    // Flags
    let flags = item.flags.to_int();
    if flags != 0 {
        dict.set("F", Object::Integer(flags));
    }

    dict
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::destination::PageDestination;

    #[test]
    fn test_outline_item_new() {
        let item = OutlineItem::new("Chapter 1");
        assert_eq!(item.title, "Chapter 1");
        assert!(item.destination.is_none());
        assert!(item.children.is_empty());
        assert!(item.color.is_none());
        assert!(!item.flags.bold);
        assert!(!item.flags.italic);
        assert!(item.open);
    }

    #[test]
    fn test_outline_item_builder() {
        let dest = Destination::fit(PageDestination::PageNumber(0));
        let item = OutlineItem::new("Bold Chapter")
            .with_destination(dest)
            .with_color(Color::rgb(1.0, 0.0, 0.0))
            .bold()
            .closed();

        assert!(item.destination.is_some());
        assert!(item.color.is_some());
        assert!(item.flags.bold);
        assert!(!item.open);
    }

    #[test]
    fn test_outline_hierarchy() {
        let mut chapter1 = OutlineItem::new("Chapter 1");
        chapter1.add_child(OutlineItem::new("Section 1.1"));
        chapter1.add_child(OutlineItem::new("Section 1.2"));

        assert_eq!(chapter1.children.len(), 2);
        assert_eq!(chapter1.count_all(), 3); // Chapter + 2 sections
    }

    #[test]
    fn test_outline_count() {
        let mut root = OutlineItem::new("Book");

        let mut ch1 = OutlineItem::new("Chapter 1");
        ch1.add_child(OutlineItem::new("Section 1.1"));
        ch1.add_child(OutlineItem::new("Section 1.2"));

        let mut ch2 = OutlineItem::new("Chapter 2").closed();
        ch2.add_child(OutlineItem::new("Section 2.1"));

        root.add_child(ch1);
        root.add_child(ch2);

        assert_eq!(root.count_all(), 6); // Book + 2 chapters + 3 sections
        assert_eq!(root.count_visible(), 5); // Ch2's child hidden
    }

    #[test]
    fn test_outline_builder() {
        let mut builder = OutlineBuilder::new();

        // Add root items
        builder.add_item(OutlineItem::new("Preface"));

        // Add chapter with sections
        builder.push_item(OutlineItem::new("Chapter 1"));
        builder.add_item(OutlineItem::new("Section 1.1"));
        builder.add_item(OutlineItem::new("Section 1.2"));
        builder.pop_item();

        builder.add_item(OutlineItem::new("Chapter 2"));

        let tree = builder.build();
        assert_eq!(tree.items.len(), 3); // Preface, Ch1, Ch2
        assert_eq!(tree.total_count(), 5); // All items
    }

    #[test]
    fn test_outline_flags() {
        let flags = OutlineFlags {
            italic: true,
            bold: true,
        };
        assert_eq!(flags.to_int(), 3);

        let flags2 = OutlineFlags {
            italic: true,
            bold: false,
        };
        assert_eq!(flags2.to_int(), 1);

        let flags3 = OutlineFlags::default();
        assert_eq!(flags3.to_int(), 0);
    }
}
