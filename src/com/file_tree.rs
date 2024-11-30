use std::{collections::HashMap, path::Path};

#[derive(Debug, Default)]
pub struct FileTree<File> {
    dirs: HashMap<String, FileTree<File>>,
    files: HashMap<String, File>,
}

impl<File> FileTree<File> {
    pub fn new() -> Self {
        Self {
            dirs: HashMap::new(),
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path: impl AsRef<Path>, file: File) {
        let mut tree = self;

        let path = path.as_ref();
        let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
            return;
        };

        for part in Path::new(parent) {
            let name = part.to_string_lossy().into_owned();
            let dir = tree.dirs.entry(name).or_insert(Self::new());
            tree = dir;
        }

        tree.files.insert(name.to_string_lossy().to_string(), file);
    }

    pub fn get_dir(&self, name: &String) -> Option<&Self> {
        self.dirs.get(name)
    }

    pub fn get_file(&self, name: &String) -> Option<&File> {
        self.files.get(name)
    }

    pub fn get_by_path(&self, path: impl AsRef<Path>) -> Option<&File> {
        let mut tree = self;

        let path = path.as_ref();
        let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
            return None;
        };

        for part in Path::new(parent) {
            let name = part.to_string_lossy().into_owned();
            tree = tree.get_dir(&name)?;
        }

        let name = name.to_string_lossy().to_string();
        tree.get_file(&name)
    }
}
