use codespan_reporting::files::SimpleFiles;

use super::{
    reporting::{Header, Report},
    Parser,
};
use std::path::{Path, PathBuf};

pub struct Compiler {
    reports: Vec<Report>,
    file_paths: Vec<PathBuf>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            reports: Vec::new(),
            file_paths: Vec::new(),
        }
    }

    pub fn add_files(&mut self, path: impl AsRef<Path>) {
        let path_display = path.as_ref().to_string_lossy().into_owned();

        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                self.reports.push(Report::error(Header::CompilerIOPath(
                    path_display,
                    err.to_string(),
                )));
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    self.reports.push(Report::error(Header::CompilerIOPath(
                        path_display.clone(),
                        err.to_string(),
                    )));
                    continue;
                }
            };

            let file_path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(err) => {
                    self.reports.push(Report::error(Header::CompilerIOFile(
                        file_path.to_string_lossy().into_owned(),
                        err.to_string(),
                    )));
                    continue;
                }
            };

            if file_type.is_dir() {
                self.add_files(file_path);
                continue;
            }

            if let Some(Some("mar")) = file_path.extension().map(|ext| ext.to_str()) {
                self.file_paths.push(file_path.to_path_buf())
            }
        }
    }

    pub fn compile(&mut self) {
        if self.file_paths.is_empty() {
            self.reports.push(Report::error(Header::CompilerNoInput()));
            return;
        }

        let mut files = SimpleFiles::new();

        let mut ast = Vec::new();
        for path in &self.file_paths {
            let file_path = path.to_string_lossy().into_owned();
            let source = match std::fs::read_to_string(path) {
                Ok(source) => source,
                Err(err) => {
                    self.reports.push(Report::error(Header::CompilerIOFile(
                        file_path.clone(),
                        err.to_string(),
                    )));
                    String::new()
                }
            };

            files.add(file_path.clone(), source.clone());
            let mut parser = Parser::new(&source, &mut self.reports);
            let expr = parser.parse_file();
            ast.push(expr);
        }
    }
}
