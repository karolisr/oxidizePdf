use crate::objects::Object;

#[derive(Debug, Clone)]
pub struct Array {
    elements: Vec<Object>,
}

impl Array {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, object: impl Into<Object>) {
        self.elements.push(object.into());
    }

    pub fn pop(&mut self) -> Option<Object> {
        self.elements.pop()
    }

    pub fn insert(&mut self, index: usize, object: impl Into<Object>) {
        self.elements.insert(index, object.into());
    }

    pub fn remove(&mut self, index: usize) -> Object {
        self.elements.remove(index)
    }

    pub fn get(&self, index: usize) -> Option<&Object> {
        self.elements.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Object> {
        self.elements.get_mut(index)
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Object> {
        self.elements.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.elements.iter_mut()
    }
}

impl Default for Array {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Object>> for Array {
    fn from(elements: Vec<Object>) -> Self {
        Self { elements }
    }
}

impl From<Array> for Vec<Object> {
    fn from(array: Array) -> Self {
        array.elements
    }
}

impl FromIterator<Object> for Array {
    fn from_iter<T: IntoIterator<Item = Object>>(iter: T) -> Self {
        Self {
            elements: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests;
