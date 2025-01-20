use std::{collections::HashMap, hash::Hash, mem};

#[derive(Debug, Clone)]
pub struct Scope<K, T>
where
    K: Eq + Hash,
{
    parent: Option<Box<Scope<K, T>>>,
    bindings: HashMap<K, T>,
    blocking: bool,
    depth: usize,
}

impl<K, T> Scope<K, T>
where
    K: Eq + Hash,
{
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

    pub fn search(&self, key: K) -> Option<&T> {
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

    pub fn insert(&mut self, key: K, val: T) -> Option<T> {
        self.bindings.insert(key, val)
    }
}
