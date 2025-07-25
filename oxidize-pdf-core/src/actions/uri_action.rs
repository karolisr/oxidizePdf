//! URI actions for opening web links

use crate::objects::{Dictionary, Object};

/// URI action flags
#[derive(Debug, Clone, Default)]
pub struct UriActionFlags {
    /// Whether coordinates are in map format
    pub is_map: bool,
}

/// URI action - resolve and open a URI
#[derive(Debug, Clone)]
pub struct UriAction {
    /// The URI to open
    pub uri: String,
    /// Action flags
    pub flags: UriActionFlags,
}

impl UriAction {
    /// Create new URI action
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            flags: UriActionFlags::default(),
        }
    }

    /// Create URI action for web URL
    pub fn web(url: impl Into<String>) -> Self {
        Self::new(url)
    }

    /// Create URI action for email
    pub fn email(address: impl Into<String>) -> Self {
        Self::new(format!("mailto:{}", address.into()))
    }

    /// Create URI action for email with subject
    pub fn email_with_subject(address: impl Into<String>, subject: impl Into<String>) -> Self {
        let encoded_subject = urlencoding::encode(&subject.into());
        Self::new(format!(
            "mailto:{}?subject={}",
            address.into(),
            encoded_subject
        ))
    }

    /// Set is_map flag
    pub fn with_map(mut self, is_map: bool) -> Self {
        self.flags.is_map = is_map;
        self
    }

    /// Convert to dictionary
    pub fn to_dict(&self) -> Dictionary {
        let mut dict = Dictionary::new();
        dict.set("Type", Object::Name("Action".to_string()));
        dict.set("S", Object::Name("URI".to_string()));
        dict.set("URI", Object::String(self.uri.clone()));

        if self.flags.is_map {
            dict.set("IsMap", Object::Boolean(true));
        }

        dict
    }
}

/// Helper to build complex URIs
#[allow(dead_code)]
pub struct UriBuilder {
    base: String,
    params: Vec<(String, String)>,
}

#[allow(dead_code)]
impl UriBuilder {
    /// Create new URI builder
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            params: Vec::new(),
        }
    }

    /// Add query parameter
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.push((key.into(), value.into()));
        self
    }

    /// Build the URI
    pub fn build(self) -> String {
        if self.params.is_empty() {
            self.base
        } else {
            let query: Vec<String> = self
                .params
                .into_iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(&k), urlencoding::encode(&v)))
                .collect();
            format!("{}?{}", self.base, query.join("&"))
        }
    }
}

/// URL encoding helper (simplified version)
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ' ' => "+".to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_action_web() {
        let action = UriAction::web("https://example.com");
        let dict = action.to_dict();

        assert_eq!(dict.get("S"), Some(&Object::Name("URI".to_string())));
        assert_eq!(
            dict.get("URI"),
            Some(&Object::String("https://example.com".to_string()))
        );
        assert!(dict.get("IsMap").is_none());
    }

    #[test]
    fn test_uri_action_email() {
        let action = UriAction::email("test@example.com");
        let dict = action.to_dict();

        assert_eq!(
            dict.get("URI"),
            Some(&Object::String("mailto:test@example.com".to_string()))
        );
    }

    #[test]
    fn test_uri_action_email_with_subject() {
        let action = UriAction::email_with_subject("test@example.com", "Hello World");
        let dict = action.to_dict();

        let uri = dict
            .get("URI")
            .and_then(|o| match o {
                Object::String(s) => Some(s),
                _ => None,
            })
            .unwrap();

        assert!(uri.starts_with("mailto:test@example.com?subject="));
        assert!(uri.contains("Hello+World") || uri.contains("Hello%20World"));
    }

    #[test]
    fn test_uri_action_with_map() {
        let action = UriAction::new("https://maps.example.com").with_map(true);
        let dict = action.to_dict();

        assert_eq!(dict.get("IsMap"), Some(&Object::Boolean(true)));
    }

    #[test]
    fn test_uri_builder() {
        let uri = UriBuilder::new("https://api.example.com/search")
            .param("q", "rust pdf")
            .param("page", "1")
            .build();

        assert!(uri.starts_with("https://api.example.com/search?"));
        assert!(uri.contains("q=rust+pdf") || uri.contains("q=rust%20pdf"));
        assert!(uri.contains("page=1"));
    }

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding::encode("Hello World"), "Hello+World");
        assert_eq!(
            urlencoding::encode("test@example.com"),
            "test%40example.com"
        );
        assert_eq!(urlencoding::encode("a-b_c.d~e"), "a-b_c.d~e");
    }
}
