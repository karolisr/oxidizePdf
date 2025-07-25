//! GoTo actions for navigating within and between documents

use crate::objects::{Dictionary, Object};
use crate::structure::{Destination, PageDestination};

/// GoTo action - navigate to a destination in the current document
#[derive(Debug, Clone)]
pub struct GoToAction {
    /// The destination
    pub destination: Destination,
}

impl GoToAction {
    /// Create new GoTo action
    pub fn new(destination: Destination) -> Self {
        Self { destination }
    }

    /// Create action to go to specific page
    pub fn to_page(page_number: u32) -> Self {
        Self {
            destination: Destination::fit(PageDestination::PageNumber(page_number)),
        }
    }

    /// Create action to go to page with zoom
    pub fn to_page_xyz(page_number: u32, x: f64, y: f64, zoom: Option<f64>) -> Self {
        Self {
            destination: Destination::xyz(
                PageDestination::PageNumber(page_number),
                Some(x),
                Some(y),
                zoom,
            ),
        }
    }

    /// Convert to dictionary
    pub fn to_dict(&self) -> Dictionary {
        let mut dict = Dictionary::new();
        dict.set("Type", Object::Name("Action".to_string()));
        dict.set("S", Object::Name("GoTo".to_string()));
        dict.set("D", Object::Array(self.destination.to_array().into()));
        dict
    }
}

/// Remote GoTo action - navigate to a destination in another PDF document
#[derive(Debug, Clone)]
pub struct RemoteGoToAction {
    /// File specification (path to the PDF)
    pub file: String,
    /// Destination in the remote document
    pub destination: Option<RemoteDestination>,
    /// Whether to open in new window
    pub new_window: Option<bool>,
}

/// Remote destination specification
#[derive(Debug, Clone)]
pub enum RemoteDestination {
    /// Page number (0-based)
    PageNumber(u32),
    /// Named destination
    Named(String),
    /// Explicit destination
    Explicit(Destination),
}

impl RemoteGoToAction {
    /// Create new remote GoTo action
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            destination: None,
            new_window: None,
        }
    }

    /// Set destination by page number
    pub fn to_page(mut self, page: u32) -> Self {
        self.destination = Some(RemoteDestination::PageNumber(page));
        self
    }

    /// Set destination by name
    pub fn to_named(mut self, name: impl Into<String>) -> Self {
        self.destination = Some(RemoteDestination::Named(name.into()));
        self
    }

    /// Set explicit destination
    pub fn to_destination(mut self, dest: Destination) -> Self {
        self.destination = Some(RemoteDestination::Explicit(dest));
        self
    }

    /// Set whether to open in new window
    pub fn in_new_window(mut self, new_window: bool) -> Self {
        self.new_window = Some(new_window);
        self
    }

    /// Convert to dictionary
    pub fn to_dict(&self) -> Dictionary {
        let mut dict = Dictionary::new();
        dict.set("Type", Object::Name("Action".to_string()));
        dict.set("S", Object::Name("GoToR".to_string()));
        dict.set("F", Object::String(self.file.clone()));

        if let Some(dest) = &self.destination {
            match dest {
                RemoteDestination::PageNumber(page) => {
                    dict.set("D", Object::Integer(*page as i64));
                }
                RemoteDestination::Named(name) => {
                    dict.set("D", Object::String(name.clone()));
                }
                RemoteDestination::Explicit(destination) => {
                    dict.set("D", Object::Array(destination.to_array().into()));
                }
            }
        }

        if let Some(nw) = self.new_window {
            dict.set("NewWindow", Object::Boolean(nw));
        }

        dict
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_goto_action_to_page() {
        let action = GoToAction::to_page(5);
        let dict = action.to_dict();

        assert_eq!(dict.get("S"), Some(&Object::Name("GoTo".to_string())));
        assert!(dict.get("D").is_some());
    }

    #[test]
    fn test_goto_action_xyz() {
        let action = GoToAction::to_page_xyz(2, 100.0, 200.0, Some(1.5));
        let dict = action.to_dict();

        assert_eq!(dict.get("S"), Some(&Object::Name("GoTo".to_string())));

        // Verify destination array
        if let Some(Object::Array(dest_array)) = dict.get("D") {
            assert!(dest_array.len() >= 5); // Page, XYZ, left, top, zoom
        } else {
            panic!("Expected destination array");
        }
    }

    #[test]
    fn test_remote_goto_action() {
        let action = RemoteGoToAction::new("other.pdf")
            .to_page(10)
            .in_new_window(true);

        let dict = action.to_dict();

        assert_eq!(dict.get("S"), Some(&Object::Name("GoToR".to_string())));
        assert_eq!(
            dict.get("F"),
            Some(&Object::String("other.pdf".to_string()))
        );
        assert_eq!(dict.get("D"), Some(&Object::Integer(10)));
        assert_eq!(dict.get("NewWindow"), Some(&Object::Boolean(true)));
    }

    #[test]
    fn test_remote_goto_named() {
        let action = RemoteGoToAction::new("document.pdf").to_named("Chapter1");

        let dict = action.to_dict();

        assert_eq!(dict.get("D"), Some(&Object::String("Chapter1".to_string())));
    }
}
