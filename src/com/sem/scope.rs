use std::{collections::HashMap, mem};

#[derive(Debug, Clone)]
pub struct Scope<'a, T> {
    parent: Option<Box<Scope<'a, T>>>,
    bindings: HashMap<&'a str, T>,
    blocking: bool,
}

impl<'a, T> Scope<'a, T> {
    fn new(blocking: bool) -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
            blocking,
        }
    }

    pub fn root() -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
            blocking: false,
        }
    }

    pub fn open(&mut self, blocking: bool) {
        let parent = mem::replace(self, Self::new(blocking));
        self.parent = Some(Box::new(parent))
    }

    pub fn close(&mut self) {
        *self = *self.parent.take().expect("scope underflow");
    }

    pub fn search(&self, key: &'a str) -> Option<&T> {
        match self.bindings.get(&key) {
            Some(value) => Some(value),
            None => match &self.parent {
                Some(parent) if !self.blocking => parent.search(key),
                _ => None,
            },
        }
    }

    pub fn insert(&mut self, key: &'a str, val: T) -> Option<T> {
        self.bindings.insert(key, val)
    }
}
