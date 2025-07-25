//! Page tree structure according to ISO 32000-1 Section 7.7.3

use crate::error::{PdfError, Result};
use crate::geometry::Rectangle;
use crate::objects::{Array, Dictionary, Object, ObjectId};
use crate::page::Page;
use std::collections::HashMap;

/// Page tree node type
#[derive(Clone)]
pub enum PageTreeNode {
    /// Pages node (intermediate node)
    Pages {
        /// Child nodes
        kids: Vec<ObjectId>,
        /// Total page count in subtree
        count: u32,
        /// Media box inherited by children
        media_box: Option<Rectangle>,
        /// Resources inherited by children
        resources: Option<Dictionary>,
        /// Parent node reference
        parent: Option<ObjectId>,
    },
    /// Page node (leaf node)
    Page {
        /// Page content
        page: Page,
        /// Parent node reference
        parent: ObjectId,
    },
}

/// Page tree structure
pub struct PageTree {
    /// Root node ID
    root: ObjectId,
    /// All nodes in the tree
    nodes: HashMap<ObjectId, PageTreeNode>,
    /// Page to node mapping
    page_map: HashMap<u32, ObjectId>,
}

impl PageTree {
    /// Create new page tree
    pub fn new(root_id: ObjectId) -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(
            root_id,
            PageTreeNode::Pages {
                kids: Vec::new(),
                count: 0,
                media_box: None,
                resources: None,
                parent: None,
            },
        );

        Self {
            root: root_id,
            nodes,
            page_map: HashMap::new(),
        }
    }

    /// Get root node ID
    pub fn root(&self) -> ObjectId {
        self.root
    }

    /// Get total page count
    pub fn page_count(&self) -> u32 {
        match self.nodes.get(&self.root) {
            Some(PageTreeNode::Pages { count, .. }) => *count,
            _ => 0,
        }
    }

    /// Get page by index
    pub fn get_page(&self, index: u32) -> Option<&Page> {
        let node_id = self.page_map.get(&index)?;
        match self.nodes.get(node_id)? {
            PageTreeNode::Page { page, .. } => Some(page),
            _ => None,
        }
    }

    /// Add a page
    pub fn add_page(&mut self, page: Page, page_id: ObjectId, parent_id: ObjectId) -> Result<()> {
        // Add page node
        self.nodes.insert(
            page_id,
            PageTreeNode::Page {
                page,
                parent: parent_id,
            },
        );

        // Update parent
        self.add_kid_to_parent(parent_id, page_id)?;

        // Update page map
        let page_index = self.page_count() - 1;
        self.page_map.insert(page_index, page_id);

        Ok(())
    }

    /// Add intermediate node
    pub fn add_pages_node(&mut self, node_id: ObjectId, parent_id: Option<ObjectId>) -> Result<()> {
        self.nodes.insert(
            node_id,
            PageTreeNode::Pages {
                kids: Vec::new(),
                count: 0,
                media_box: None,
                resources: None,
                parent: parent_id,
            },
        );

        if let Some(parent) = parent_id {
            self.add_kid_to_parent(parent, node_id)?;
        }

        Ok(())
    }

    /// Add kid to parent node
    fn add_kid_to_parent(&mut self, parent_id: ObjectId, kid_id: ObjectId) -> Result<()> {
        match self.nodes.get_mut(&parent_id) {
            Some(PageTreeNode::Pages { kids, count, .. }) => {
                kids.push(kid_id);
                *count += 1;
                self.update_ancestor_counts(parent_id)?;
                Ok(())
            }
            _ => Err(PdfError::InvalidStructure(
                "Parent is not a Pages node".to_string(),
            )),
        }
    }

    /// Update ancestor page counts
    fn update_ancestor_counts(&mut self, node_id: ObjectId) -> Result<()> {
        if let Some(PageTreeNode::Pages { parent, .. }) = self.nodes.get(&node_id) {
            if let Some(parent_id) = parent {
                let parent_id = *parent_id;
                if let Some(PageTreeNode::Pages { count, .. }) = self.nodes.get_mut(&parent_id) {
                    *count += 1;
                    self.update_ancestor_counts(parent_id)?;
                }
            }
        }
        Ok(())
    }

    /// Convert node to dictionary
    pub fn node_to_dict(&self, node_id: ObjectId) -> Result<Dictionary> {
        let node = self
            .nodes
            .get(&node_id)
            .ok_or_else(|| PdfError::InvalidStructure("Node not found".to_string()))?;

        match node {
            PageTreeNode::Pages {
                kids,
                count,
                media_box,
                resources,
                parent,
            } => {
                let mut dict = Dictionary::new();
                dict.set("Type", Object::Name("Pages".to_string()));

                // Kids array
                let kids_array: Array = kids.iter().map(|id| Object::Reference(*id)).collect();
                dict.set("Kids", Object::Array(kids_array.into()));

                dict.set("Count", Object::Integer(*count as i64));

                if let Some(parent_id) = parent {
                    dict.set("Parent", Object::Reference(*parent_id));
                }

                if let Some(mb) = media_box {
                    let mb_array = Array::from(vec![
                        Object::Real(mb.lower_left.x),
                        Object::Real(mb.lower_left.y),
                        Object::Real(mb.upper_right.x),
                        Object::Real(mb.upper_right.y),
                    ]);
                    dict.set("MediaBox", Object::Array(mb_array.into()));
                }

                if let Some(res) = resources {
                    dict.set("Resources", Object::Dictionary(res.clone()));
                }

                Ok(dict)
            }
            PageTreeNode::Page { page, parent } => {
                // Convert page to dictionary
                let mut dict = page.to_dict();
                dict.set("Type", Object::Name("Page".to_string()));
                dict.set("Parent", Object::Reference(*parent));
                Ok(dict)
            }
        }
    }
}

/// Page tree builder
pub struct PageTreeBuilder {
    /// Current tree
    tree: PageTree,
    /// Next object ID
    next_id: u32,
}

impl PageTreeBuilder {
    /// Create new builder
    pub fn new(start_id: u32) -> Self {
        let root_id = ObjectId::new(start_id, 0);
        Self {
            tree: PageTree::new(root_id),
            next_id: start_id + 1,
        }
    }

    /// Add pages in balanced tree structure
    pub fn add_pages(&mut self, pages: Vec<Page>) -> Result<()> {
        // For simplicity, add all pages under root
        // In production, create intermediate nodes for better performance
        for page in pages {
            let page_id = ObjectId::new(self.next_id, 0);
            self.next_id += 1;
            self.tree.add_page(page, page_id, self.tree.root)?;
        }
        Ok(())
    }

    /// Build the tree
    pub fn build(self) -> PageTree {
        self.tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_tree_new() {
        let root_id = ObjectId::new(1, 0);
        let tree = PageTree::new(root_id);
        assert_eq!(tree.root(), root_id);
        assert_eq!(tree.page_count(), 0);
    }

    #[test]
    fn test_add_page() {
        let root_id = ObjectId::new(1, 0);
        let mut tree = PageTree::new(root_id);

        let page = Page::a4();
        let page_id = ObjectId::new(2, 0);

        tree.add_page(page, page_id, root_id).unwrap();
        assert_eq!(tree.page_count(), 1);
        assert!(tree.get_page(0).is_some());
    }

    #[test]
    fn test_page_tree_builder() {
        let mut builder = PageTreeBuilder::new(1);
        let pages = vec![Page::a4(), Page::letter(), Page::a4()];

        builder.add_pages(pages).unwrap();
        let tree = builder.build();

        assert_eq!(tree.page_count(), 3);
        assert!(tree.get_page(0).is_some());
        assert!(tree.get_page(1).is_some());
        assert!(tree.get_page(2).is_some());
        assert!(tree.get_page(3).is_none());
    }

    #[test]
    fn test_node_to_dict() {
        let root_id = ObjectId::new(1, 0);
        let tree = PageTree::new(root_id);

        let dict = tree.node_to_dict(root_id).unwrap();
        assert_eq!(dict.get("Type"), Some(&Object::Name("Pages".to_string())));
        assert!(dict.get("Kids").is_some());
        assert_eq!(dict.get("Count"), Some(&Object::Integer(0)));
    }
}
