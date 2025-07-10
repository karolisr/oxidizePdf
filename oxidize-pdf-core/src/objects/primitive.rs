use crate::objects::Dictionary;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId {
    number: u32,
    generation: u16,
}

impl ObjectId {
    pub fn new(number: u32, generation: u16) -> Self {
        Self { number, generation }
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn generation(&self) -> u16 {
        self.generation
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} R", self.number, self.generation)
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
    Name(String),
    Array(Vec<Object>),
    Dictionary(Dictionary),
    Stream(Dictionary, Vec<u8>),
    Reference(ObjectId),
}

impl Object {
    pub fn is_null(&self) -> bool {
        matches!(self, Object::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Object::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Object::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_real(&self) -> Option<f64> {
        match self {
            Object::Real(f) => Some(*f),
            Object::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Object::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        match self {
            Object::Name(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Object>> {
        match self {
            Object::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&Dictionary> {
        match self {
            Object::Dictionary(dict) => Some(dict),
            _ => None,
        }
    }
}

impl From<bool> for Object {
    fn from(b: bool) -> Self {
        Object::Boolean(b)
    }
}

impl From<i32> for Object {
    fn from(i: i32) -> Self {
        Object::Integer(i as i64)
    }
}

impl From<i64> for Object {
    fn from(i: i64) -> Self {
        Object::Integer(i)
    }
}

impl From<f32> for Object {
    fn from(f: f32) -> Self {
        Object::Real(f as f64)
    }
}

impl From<f64> for Object {
    fn from(f: f64) -> Self {
        Object::Real(f)
    }
}

impl From<String> for Object {
    fn from(s: String) -> Self {
        Object::String(s)
    }
}

impl From<&str> for Object {
    fn from(s: &str) -> Self {
        Object::String(s.to_string())
    }
}

impl From<Vec<Object>> for Object {
    fn from(v: Vec<Object>) -> Self {
        Object::Array(v)
    }
}

impl From<Dictionary> for Object {
    fn from(d: Dictionary) -> Self {
        Object::Dictionary(d)
    }
}
