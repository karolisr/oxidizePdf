//! Base action types and dictionary

use crate::objects::{Dictionary, Object, ObjectId};
use crate::structure::Destination;

/// PDF action types
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    /// Go to a destination in the current document
    GoTo,
    /// Go to a destination in another document
    GoToR,
    /// Go to a destination in an embedded file
    GoToE,
    /// Launch an application
    Launch,
    /// Execute a predefined action
    Named,
    /// Resolve a URI
    URI,
    /// Submit form data
    SubmitForm,
    /// Reset form fields
    ResetForm,
    /// Import form data
    ImportData,
    /// Execute JavaScript
    JavaScript,
    /// Set OCG state
    SetOCGState,
    /// Play a sound
    Sound,
    /// Play a movie
    Movie,
    /// Control multimedia presentation
    Rendition,
    /// Transition action
    Trans,
    /// 3D view action
    GoTo3DView,
}

impl ActionType {
    /// Convert to PDF name
    pub fn to_name(&self) -> &'static str {
        match self {
            ActionType::GoTo => "GoTo",
            ActionType::GoToR => "GoToR",
            ActionType::GoToE => "GoToE",
            ActionType::Launch => "Launch",
            ActionType::Named => "Named",
            ActionType::URI => "URI",
            ActionType::SubmitForm => "SubmitForm",
            ActionType::ResetForm => "ResetForm",
            ActionType::ImportData => "ImportData",
            ActionType::JavaScript => "JavaScript",
            ActionType::SetOCGState => "SetOCGState",
            ActionType::Sound => "Sound",
            ActionType::Movie => "Movie",
            ActionType::Rendition => "Rendition",
            ActionType::Trans => "Trans",
            ActionType::GoTo3DView => "GoTo3DView",
        }
    }
}

/// PDF action
#[derive(Debug, Clone)]
pub enum Action {
    /// Go to destination
    GoTo {
        /// Destination
        destination: Destination,
    },
    /// Go to remote destination
    GoToR {
        /// File specification
        file: String,
        /// Destination in the file
        destination: Option<Destination>,
        /// Whether to open in new window
        new_window: Option<bool>,
    },
    /// URI action
    URI {
        /// URI to resolve
        uri: String,
        /// Whether URI is a map
        is_map: bool,
    },
    /// Named action
    Named {
        /// Action name
        name: String,
    },
    /// Launch application
    Launch {
        /// Application/document to launch
        file: String,
        /// Parameters
        parameters: Option<String>,
        /// Whether to open in new window
        new_window: Option<bool>,
    },
    /// Next action in sequence
    Next(Box<Action>),
}

impl Action {
    /// Create GoTo action
    pub fn goto(destination: Destination) -> Self {
        Action::GoTo { destination }
    }

    /// Create URI action
    pub fn uri(uri: impl Into<String>) -> Self {
        Action::URI {
            uri: uri.into(),
            is_map: false,
        }
    }

    /// Create Named action
    pub fn named(name: impl Into<String>) -> Self {
        Action::Named { name: name.into() }
    }

    /// Create GoToR action
    pub fn goto_remote(file: impl Into<String>, destination: Option<Destination>) -> Self {
        Action::GoToR {
            file: file.into(),
            destination,
            new_window: None,
        }
    }

    /// Create Launch action
    pub fn launch(file: impl Into<String>) -> Self {
        Action::Launch {
            file: file.into(),
            parameters: None,
            new_window: None,
        }
    }

    /// Convert to dictionary
    pub fn to_dict(&self) -> Dictionary {
        let mut dict = Dictionary::new();

        match self {
            Action::GoTo { destination } => {
                dict.set("Type", Object::Name("Action".to_string()));
                dict.set("S", Object::Name("GoTo".to_string()));
                dict.set("D", Object::Array(destination.to_array().into()));
            }
            Action::GoToR {
                file,
                destination,
                new_window,
            } => {
                dict.set("Type", Object::Name("Action".to_string()));
                dict.set("S", Object::Name("GoToR".to_string()));
                dict.set("F", Object::String(file.clone()));

                if let Some(dest) = destination {
                    dict.set("D", Object::Array(dest.to_array().into()));
                }

                if let Some(nw) = new_window {
                    dict.set("NewWindow", Object::Boolean(*nw));
                }
            }
            Action::URI { uri, is_map } => {
                dict.set("Type", Object::Name("Action".to_string()));
                dict.set("S", Object::Name("URI".to_string()));
                dict.set("URI", Object::String(uri.clone()));

                if *is_map {
                    dict.set("IsMap", Object::Boolean(true));
                }
            }
            Action::Named { name } => {
                dict.set("Type", Object::Name("Action".to_string()));
                dict.set("S", Object::Name("Named".to_string()));
                dict.set("N", Object::Name(name.clone()));
            }
            Action::Launch {
                file,
                parameters,
                new_window,
            } => {
                dict.set("Type", Object::Name("Action".to_string()));
                dict.set("S", Object::Name("Launch".to_string()));
                dict.set("F", Object::String(file.clone()));

                if let Some(params) = parameters {
                    dict.set("P", Object::String(params.clone()));
                }

                if let Some(nw) = new_window {
                    dict.set("NewWindow", Object::Boolean(*nw));
                }
            }
            Action::Next(next) => {
                let next_dict = next.to_dict();
                dict = next_dict;
            }
        }

        dict
    }
}

/// Action dictionary wrapper
pub struct ActionDictionary {
    /// The action
    pub action: Action,
    /// Object ID if indirect
    pub object_id: Option<ObjectId>,
}

impl ActionDictionary {
    /// Create new action dictionary
    pub fn new(action: Action) -> Self {
        Self {
            action,
            object_id: None,
        }
    }

    /// Set object ID for indirect reference
    pub fn with_object_id(mut self, id: ObjectId) -> Self {
        self.object_id = Some(id);
        self
    }

    /// Convert to dictionary
    pub fn to_dict(&self) -> Dictionary {
        self.action.to_dict()
    }

    /// Get as object (direct or indirect reference)
    pub fn to_object(&self) -> Object {
        if let Some(id) = self.object_id {
            Object::Reference(id)
        } else {
            Object::Dictionary(self.to_dict())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::PageDestination;

    #[test]
    fn test_action_type_names() {
        assert_eq!(ActionType::GoTo.to_name(), "GoTo");
        assert_eq!(ActionType::URI.to_name(), "URI");
        assert_eq!(ActionType::Named.to_name(), "Named");
        assert_eq!(ActionType::Launch.to_name(), "Launch");
    }

    #[test]
    fn test_goto_action() {
        let dest = Destination::fit(PageDestination::PageNumber(0));
        let action = Action::goto(dest);

        let dict = action.to_dict();
        assert_eq!(dict.get("S"), Some(&Object::Name("GoTo".to_string())));
        assert!(dict.get("D").is_some());
    }

    #[test]
    fn test_uri_action() {
        let action = Action::uri("https://example.com");
        let dict = action.to_dict();

        assert_eq!(dict.get("S"), Some(&Object::Name("URI".to_string())));
        assert_eq!(
            dict.get("URI"),
            Some(&Object::String("https://example.com".to_string()))
        );
    }

    #[test]
    fn test_named_action() {
        let action = Action::named("NextPage");
        let dict = action.to_dict();

        assert_eq!(dict.get("S"), Some(&Object::Name("Named".to_string())));
        assert_eq!(dict.get("N"), Some(&Object::Name("NextPage".to_string())));
    }

    #[test]
    fn test_action_dictionary() {
        let action = Action::uri("https://example.com");
        let action_dict = ActionDictionary::new(action).with_object_id(ObjectId::new(10, 0));

        match action_dict.to_object() {
            Object::Reference(id) => {
                assert_eq!(id.number(), 10);
                assert_eq!(id.generation(), 0);
            }
            _ => panic!("Expected reference"),
        }
    }
}
