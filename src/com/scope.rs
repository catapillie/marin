use std::{collections::HashMap, mem};

#[derive(Debug, Clone)]
pub struct Scope<'a, T> {
    parent: Option<Box<Scope<'a, T>>>,
    bindings: HashMap<&'a str, T>,
    blocking: bool,
    depth: usize,
}

impl<'a, T> Scope<'a, T> {
    fn new(blocking: bool, depth: usize) -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
            blocking,
            depth,
        }
    }

    pub fn root() -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
            blocking: false,
            depth: 0,
        }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn open(&mut self, blocking: bool) {
        let parent = mem::replace(self, Self::new(blocking, self.depth + 1));
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

    pub fn find<P>(&self, predicate: P) -> Option<&T>
    where
        P: Fn(&T) -> bool,
    {
        match self.bindings.values().find(|t| predicate(t)) {
            Some(value) => Some(value),
            None => match &self.parent {
                Some(parent) if !self.blocking => parent.find(predicate),
                _ => None,
            },
        }
    }

    pub fn insert(&mut self, key: &'a str, val: T) -> Option<T> {
        self.bindings.insert(key, val)
    }
}
