use std::{
    collections::{HashMap, hash_map},
    hash::Hash,
    mem,
};

#[derive(Debug, Clone)]
pub struct Scope<K, I, T>
where
    K: Eq + Hash,
    I: Default,
{
    parent: Option<Box<Scope<K, I, T>>>,
    bindings: HashMap<K, T>,
    info: I,
    blocking: bool,
    depth: usize,
}

impl<K, I, T> Scope<K, I, T>
where
    K: Eq + Hash,
    I: Default,
{
    fn new(blocking: bool, depth: usize) -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
            info: I::default(),
            blocking,
            depth,
        }
    }

    pub fn root() -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
            info: I::default(),
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

    pub fn iter(&'_ self) -> hash_map::Iter<'_, K, T> {
        self.bindings.iter()
    }

    pub fn infos_mut(&mut self) -> &mut I {
        &mut self.info
    }

    pub fn infos_iter(&'_ self) -> ScopeInfoIterator<'_, K, I, T> {
        ScopeInfoIterator { scope: Some(self) }
    }
}

#[allow(dead_code)]
pub struct ScopeIterator<'a, K, I, T>
where
    K: Eq + Hash,
    I: Default,
{
    iter: hash_map::Iter<'a, K, T>,
    next_scope: Option<&'a Scope<K, I, T>>,
}

impl<'a, K, I, T> Iterator for ScopeIterator<'a, K, I, T>
where
    K: Eq + Hash,
    I: Default,
{
    type Item = (&'a K, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.iter.next() {
            return Some(item);
        }

        let scope = self.next_scope?;
        self.iter = scope.bindings.iter();
        self.next_scope = match &scope.parent {
            Some(parent) => Some(parent),
            None => None,
        };

        self.next()
    }
}

pub struct ScopeInfoIterator<'a, K, I, T>
where
    K: Eq + Hash,
    I: Default,
{
    scope: Option<&'a Scope<K, I, T>>,
}

impl<'a, K, I, T> Iterator for ScopeInfoIterator<'a, K, I, T>
where
    K: Eq + Hash,
    I: Default,
{
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        let scope = self.scope?;
        let i = Some(&scope.info);
        self.scope = scope.parent.as_deref();
        i
    }
}
