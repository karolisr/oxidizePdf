//! Name tree structure according to ISO 32000-1 Section 7.9.6

use crate::error::{PdfError, Result};
use crate::objects::{Array, Dictionary, Object, ObjectId};
use std::collections::BTreeMap;

/// Name tree node
#[derive(Debug, Clone)]
pub struct NameTreeNode {
    /// Names in this node (leaf node)
    names: Option<BTreeMap<String, Object>>,
    /// Child nodes (intermediate node)
    kids: Option<Vec<ObjectId>>,
    /// Limits [min, max] for this node
    limits: Option<(String, String)>,
}

impl NameTreeNode {
    /// Create leaf node
    pub fn leaf(names: BTreeMap<String, Object>) -> Self {
        let limits = if names.is_empty() {
            None
        } else {
            let min = names.keys().next().unwrap().clone();
            let max = names.keys().last().unwrap().clone();
            Some((min, max))
        };

        Self {
            names: Some(names),
            kids: None,
            limits,
        }
    }

    /// Create intermediate node
    pub fn intermediate(kids: Vec<ObjectId>, limits: (String, String)) -> Self {
        Self {
            names: None,
            kids: Some(kids),
            limits: Some(limits),
        }
    }

    /// Convert to dictionary
    pub fn to_dict(&self) -> Dictionary {
        let mut dict = Dictionary::new();

        if let Some(names) = &self.names {
            // Leaf node - create Names array
            let mut names_array = Array::new();
            for (key, value) in names {
                names_array.push(Object::String(key.clone()));
                names_array.push(value.clone());
            }
            dict.set("Names", Object::Array(names_array.into()));
        }

        if let Some(kids) = &self.kids {
            // Intermediate node - create Kids array
            let kids_array: Array = kids.iter().map(|id| Object::Reference(*id)).collect();
            dict.set("Kids", Object::Array(kids_array.into()));
        }

        if let Some((min, max)) = &self.limits {
            let limits_array = Array::from(vec![
                Object::String(min.clone()),
                Object::String(max.clone()),
            ]);
            dict.set("Limits", Object::Array(limits_array.into()));
        }

        dict
    }
}

/// Name tree structure
pub struct NameTree {
    /// Root node
    root: NameTreeNode,
    /// All nodes (for complex trees)
    #[allow(dead_code)]
    nodes: BTreeMap<ObjectId, NameTreeNode>,
}

impl Default for NameTree {
    fn default() -> Self {
        Self::new()
    }
}

impl NameTree {
    /// Create new name tree with single leaf node
    pub fn new() -> Self {
        Self {
            root: NameTreeNode::leaf(BTreeMap::new()),
            nodes: BTreeMap::new(),
        }
    }

    /// Add name-value pair
    pub fn add(&mut self, name: String, value: Object) {
        if let Some(names) = &mut self.root.names {
            names.insert(name.clone(), value);

            // Update limits
            if let Some((min, max)) = &mut self.root.limits {
                if name < *min {
                    *min = name.clone();
                }
                if name > *max {
                    *max = name;
                }
            } else {
                self.root.limits = Some((name.clone(), name));
            }
        }
    }

    /// Get value by name
    pub fn get(&self, name: &str) -> Option<&Object> {
        self.root.names.as_ref()?.get(name)
    }

    /// Convert to dictionary
    pub fn to_dict(&self) -> Dictionary {
        self.root.to_dict()
    }

    /// Create from existing dictionary
    pub fn from_dict(dict: &Dictionary) -> Result<Self> {
        let mut tree = Self::new();

        if let Some(Object::Array(names_array)) = dict.get("Names") {
            let items: Vec<&Object> = names_array.iter().collect();

            if items.len() % 2 != 0 {
                return Err(PdfError::InvalidStructure(
                    "Names array must have even length".to_string(),
                ));
            }

            for i in (0..items.len()).step_by(2) {
                if let (Object::String(key), value) = (items[i], items[i + 1]) {
                    let key = key.clone();
                    tree.add(key, value.clone());
                }
            }
        }

        Ok(tree)
    }
}

/// Named destinations
pub struct NamedDestinations {
    /// Name tree for destinations
    tree: NameTree,
}

impl Default for NamedDestinations {
    fn default() -> Self {
        Self::new()
    }
}

impl NamedDestinations {
    /// Create new named destinations
    pub fn new() -> Self {
        Self {
            tree: NameTree::new(),
        }
    }

    /// Add named destination
    pub fn add_destination(&mut self, name: String, destination: Array) {
        self.tree.add(name, Object::Array(destination.into()));
    }

    /// Get destination by name
    pub fn get_destination(&self, name: &str) -> Option<Array> {
        match self.tree.get(name)? {
            Object::Array(arr) => Some(Array::from(arr.clone())),
            _ => None,
        }
    }

    /// Convert to dictionary
    pub fn to_dict(&self) -> Dictionary {
        self.tree.to_dict()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_tree_node_leaf() {
        let mut names = BTreeMap::new();
        names.insert("First".to_string(), Object::Integer(1));
        names.insert("Second".to_string(), Object::Integer(2));

        let node = NameTreeNode::leaf(names);
        assert!(node.names.is_some());
        assert!(node.kids.is_none());
        assert_eq!(
            node.limits,
            Some(("First".to_string(), "Second".to_string()))
        );
    }

    #[test]
    fn test_name_tree_add() {
        let mut tree = NameTree::new();
        tree.add("Apple".to_string(), Object::Integer(1));
        tree.add("Banana".to_string(), Object::Integer(2));
        tree.add("Cherry".to_string(), Object::Integer(3));

        assert_eq!(tree.get("Banana"), Some(&Object::Integer(2)));
        assert_eq!(
            tree.root.limits,
            Some(("Apple".to_string(), "Cherry".to_string()))
        );
    }

    #[test]
    fn test_name_tree_to_dict() {
        let mut tree = NameTree::new();
        tree.add("Test".to_string(), Object::Boolean(true));

        let dict = tree.to_dict();
        assert!(dict.get("Names").is_some());
    }

    #[test]
    fn test_named_destinations() {
        use crate::structure::destination::{Destination, PageDestination};

        let mut dests = NamedDestinations::new();
        let dest = Destination::fit(PageDestination::PageNumber(0));
        dests.add_destination("Home".to_string(), dest.to_array());

        let retrieved = dests.get_destination("Home");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_name_tree_from_dict() {
        let mut dict = Dictionary::new();
        let names_array = Array::from(vec![
            Object::String("First".to_string()),
            Object::Integer(1),
            Object::String("Second".to_string()),
            Object::Integer(2),
        ]);
        dict.set("Names", Object::Array(names_array.into()));

        let tree = NameTree::from_dict(&dict).unwrap();
        assert_eq!(tree.get("First"), Some(&Object::Integer(1)));
        assert_eq!(tree.get("Second"), Some(&Object::Integer(2)));
    }
}
