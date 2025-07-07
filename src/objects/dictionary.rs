use crate::objects::Object;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Dictionary {
    entries: HashMap<String, Object>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
        }
    }
    
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Object>) {
        self.entries.insert(key.into(), value.into());
    }
    
    pub fn get(&self, key: &str) -> Option<&Object> {
        self.entries.get(key)
    }
    
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Object> {
        self.entries.get_mut(key)
    }
    
    pub fn remove(&mut self, key: &str) -> Option<Object> {
        self.entries.remove(key)
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }
    
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }
    
    pub fn values(&self) -> impl Iterator<Item = &Object> {
        self.entries.values()
    }
    
    pub fn entries(&self) -> impl Iterator<Item = (&String, &Object)> {
        self.entries.iter()
    }
    
    pub fn entries_mut(&mut self) -> impl Iterator<Item = (&String, &mut Object)> {
        self.entries.iter_mut()
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<(String, Object)> for Dictionary {
    fn from_iter<T: IntoIterator<Item = (String, Object)>>(iter: T) -> Self {
        let mut dict = Dictionary::new();
        for (key, value) in iter {
            dict.set(key, value);
        }
        dict
    }
}