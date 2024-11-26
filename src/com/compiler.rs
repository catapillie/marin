use super::{
    ast,
    reporting::{Header, Report},
    Parser,
};
use codespan_reporting::{
    files::{self, SimpleFile},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use std::path::{Path, PathBuf};

pub struct Compiler<Stage> {
    reports: Vec<Report>,
    files: Files<Stage>,
}

// compiler stage
pub struct Staged(PathBuf);
pub struct Source;
pub struct Parsed(ast::File);

// compiler initialization
pub fn init() -> Compiler<Staged> {
    Compiler {
        reports: Vec::new(),
        files: Files::default(),
    }
}

impl<T> Compiler<T> {
    pub fn emit_reports(&self, color: ColorChoice, config: &Config) -> Result<(), files::Error> {
        let writer = StandardStream::stderr(color);
        for report in &self.reports {
            term::emit(
                &mut writer.lock(),
                config,
                &self.files,
                &report.to_diagnostic(),
            )?;
        }
        Ok(())
    }
}

impl Compiler<Staged> {
    fn check_extension(path: &Path) -> bool {
        matches!(path.extension().map(|ext| ext.to_str()), Some(Some("mar")))
    }

    fn add_dir(&mut self, path: &Path) {
        let path_display = path.to_string_lossy().into_owned();

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
                self.add_dir(&file_path);
                continue;
            }

            if Self::check_extension(&file_path) {
                self.files
                    .0
                    .push((File::new(String::new(), String::new()), Staged(file_path)))
            }
        }
    }

    pub fn add_files(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        let path_display = path.to_string_lossy().into_owned();

        if !path.exists() {
            self.reports
                .push(Report::error(Header::CompilerNoSuchPath(path_display)));
            return;
        }

        if path.is_dir() {
            self.add_dir(path);
            return;
        }

        if Self::check_extension(path) {
            self.files.0.push((
                File::new(String::new(), String::new()),
                Staged(path.to_path_buf()),
            ))
        }
    }

    fn read_file(staged: &Staged, reports: &mut Vec<Report>) -> File {
        let path = &staged.0;
        let file_path = path.to_string_lossy().into_owned();
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(err) => {
                reports.push(Report::error(Header::CompilerIOFile(
                    file_path.clone(),
                    err.to_string(),
                )));
                String::new()
            }
        };

        File::new(file_path, source)
    }

    pub fn read_sources(mut self) -> Compiler<Source> {
        if self.files.0.is_empty() {
            self.reports.push(Report::error(Header::CompilerNoInput()));
        }

        let mut reports = Vec::new();
        let source_files = self
            .files
            .0
            .iter()
            .map(|(_, f)| (Self::read_file(f, &mut reports), Source))
            .collect();
        self.reports.append(&mut reports);

        Compiler {
            reports: self.reports,
            files: Files(source_files),
        }
    }
}

impl Compiler<Source> {
    fn parse_file(f: &File, id: usize, reports: &mut Vec<Report>) -> Parsed {
        let mut parser = Parser::new(f.source(), id, reports);
        Parsed(parser.parse_file())
    }

    pub fn parse(mut self) -> Compiler<Parsed> {
        let mut reports = Vec::new();
        let parsed_files = self
            .files
            .0
            .into_iter()
            .enumerate()
            .map(|(id, (f, _))| {
                let parsed = Self::parse_file(&f, id, &mut reports);
                (f, parsed)
            })
            .collect();
        self.reports.append(&mut reports);

        Compiler {
            reports: self.reports,
            files: Files(parsed_files),
        }
    }
}

type File = SimpleFile<String, String>;
struct Files<T>(Vec<(File, T)>);

impl<T> Default for Files<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<'a, T> files::Files<'a> for Files<T> {
    type FileId = usize;
    type Name = &'a str;
    type Source = &'a str;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, files::Error> {
        match self.0.get(id) {
            Some((file, _)) => Ok(file.name()),
            None => Err(files::Error::FileMissing),
        }
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, files::Error> {
        match self.0.get(id) {
            Some((file, _)) => Ok(file.source()),
            None => Err(files::Error::FileMissing),
        }
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, files::Error> {
        match self.0.get(id) {
            Some((file, _)) => file.line_index((), byte_index),
            None => Err(files::Error::FileMissing),
        }
    }

    fn line_range(
        &'a self,
        id: Self::FileId,
        line_index: usize,
    ) -> Result<std::ops::Range<usize>, files::Error> {
        match self.0.get(id) {
            Some((file, _)) => file.line_range((), line_index),
            None => Err(files::Error::FileMissing),
        }
    }
}
