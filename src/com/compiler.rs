use super::{
    ast, ir,
    reporting::{Header, Report},
    sem::{self},
    Checker, Parser,
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

pub struct Compiler<Stage, Info> {
    reports: Vec<Report>,
    files: Files<Stage>,
    info: Info,
}

// compiler stage
#[derive(Default, Clone)]
pub struct StagedFileInfo {
    is_from_std: bool,
}
impl StagedFileInfo {
    pub fn marin_std_file() -> Self {
        Self { is_from_std: true }
    }
}
pub struct Staged(PathBuf, StagedFileInfo);

pub struct Sourced(StagedFileInfo);

pub struct Parsed(pub ast::File, pub StagedFileInfo);

pub struct Checked(pub ir::Module);

// compiler info
pub struct StagedInfo {
    staged_std: bool,
}

pub struct SourceInfo;

pub struct ParsedInfo;

pub struct CheckedInfo {
    pub evaluation_order: Vec<usize>,
}

// compiler initialization
pub fn init() -> Compiler<Staged, StagedInfo> {
    Compiler {
        reports: Vec::new(),
        files: Files::default(),
        info: StagedInfo { staged_std: false },
    }
}

impl<T, I> Compiler<T, I> {
    pub fn file_contents(&self) -> Box<[&T]> {
        self.files.0.iter().map(|(_, _, t)| t).collect()
    }

    pub fn info(&self) -> &I {
        &self.info
    }

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

impl Compiler<Staged, StagedInfo> {
    // pub fn add_dir(&mut self, path: impl AsRef<Path>) {
    //     self.add_dir_with_info(path, StagedFileInfo::default());
    // }

    fn add_dir_with_info(&mut self, path: impl AsRef<Path>, info: StagedFileInfo) {
        let path = path.as_ref();
        let path_display = path.display().to_string();

        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                self.reports.push(Report::error(Header::CompilerIO(
                    path_display.to_string(),
                    err.to_string(),
                )));
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    self.reports.push(Report::error(Header::CompilerIO(
                        path_display.to_string(),
                        err.to_string(),
                    )));
                    continue;
                }
            };

            let entry_path = entry.path();
            let entry_type = match entry.file_type() {
                Ok(kind) => kind,
                Err(err) => {
                    self.reports.push(Report::error(Header::CompilerIO(
                        entry_path.display().to_string(),
                        err.to_string(),
                    )));
                    continue;
                }
            };

            if entry_type.is_dir() {
                self.add_dir_with_info(&entry_path, info.clone());
                continue;
            }

            if entry_type.is_file() {
                self.add_file_with_info(&entry_path, info.clone());
                continue;
            }
        }
    }

    pub fn add_file(&mut self, path: impl AsRef<Path>) {
        self.add_file_with_info(path, StagedFileInfo::default());
    }

    fn add_file_with_info(&mut self, path: impl AsRef<Path>, info: StagedFileInfo) {
        let path = path.as_ref();
        let path_display = path.display().to_string();

        if !path.exists() {
            self.reports
                .push(Report::error(Header::CompilerNoSuchPath(path_display)));
            return;
        }

        if !matches!(path.extension().map(|s| s.to_str()), Some(Some("mar"))) {
            self.reports
                .push(Report::error(Header::CompilerBadExtension(path_display)));
            return;
        }

        let full_path = path.canonicalize().expect("cannot canonicalize file path");

        self.files.0.push((
            File::new(String::new(), String::new()),
            full_path,
            Staged(path.to_path_buf(), info),
        ))
    }

    pub fn add_marin_std(&mut self) {
        if self.info.staged_std {
            panic!("marin std library is already staged");
        }

        let marin_std_path = std::env::current_exe()
            .expect("need access to current executable directory")
            .parent()
            .unwrap()
            .join("std");
        self.add_dir_with_info(&marin_std_path, StagedFileInfo::marin_std_file());
        self.info.staged_std = true;
    }

    fn read_file(staged: &Staged, reports: &mut Vec<Report>) -> File {
        let path = &staged.0;
        let file_path = path.display().to_string();
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(err) => {
                reports.push(Report::error(Header::CompilerIO(
                    file_path.clone(),
                    err.to_string(),
                )));
                String::new()
            }
        };

        File::new(file_path, source)
    }

    pub fn read_sources(mut self) -> Compiler<Sourced, SourceInfo> {
        if self.files.0.is_empty() {
            self.reports.push(Report::error(Header::CompilerNoInput()));
        }

        let mut reports = Vec::new();
        let source_files = self
            .files
            .0
            .into_iter()
            .map(|(_, p, f)| (Self::read_file(&f, &mut reports), p, Sourced(f.1)))
            .collect();
        self.reports.append(&mut reports);

        Compiler {
            reports: self.reports,
            files: Files(source_files),
            info: SourceInfo,
        }
    }
}

impl Compiler<Sourced, SourceInfo> {
    fn parse_file(
        file: &File,
        id: usize,
        info: StagedFileInfo,
        reports: &mut Vec<Report>,
    ) -> Parsed {
        let mut parser = Parser::new(file.source(), id, reports);
        Parsed(parser.parse_file(), info)
    }

    pub fn parse(mut self) -> Compiler<Parsed, ParsedInfo> {
        let mut reports = Vec::new();
        let parsed_files = self
            .files
            .0
            .into_iter()
            .enumerate()
            .map(|(id, (f, p, Sourced(info)))| {
                let parsed = Self::parse_file(&f, id, info, &mut reports);
                (f, p, parsed)
            })
            .collect();
        self.reports.append(&mut reports);

        Compiler {
            reports: self.reports,
            files: Files(parsed_files),
            info: ParsedInfo,
        }
    }
}

impl Compiler<Parsed, ParsedInfo> {
    pub fn check(mut self) -> Compiler<Checked, CheckedInfo> {
        let deps = sem::build_dependency_graph(&self.files, &mut self.reports);
        let order = sem::sort_dependencies(&deps, &self.files, &mut self.reports);

        let files = self.files.0;
        let mut irs = Vec::new();
        irs.resize_with(files.len(), || None);

        let mut reports = Vec::new();
        let mut checker = Checker::new(files.len(), &deps, &mut reports);

        for scc in &order {
            for id in scc {
                let id = *id;
                let (file, _, Parsed(ast, info)) = &files[id];

                let options = sem::CheckModuleOptions::new().set_verbose(!info.is_from_std);
                let ir = checker.check_module(file.name(), id, file.source(), ast, options);
                irs[id] = Some(Checked(ir))
            }
        }
        self.reports.append(&mut reports);

        let checked_files = files
            .into_iter()
            .zip(irs)
            .map(|((file, path, _), ir)| {
                let ir =
                    ir.unwrap_or_else(|| panic!("file '{}' was left unchecked", &path.display()));
                (file, path, ir)
            })
            .collect();

        let evaluation_order = order.into_iter().flatten().collect();

        Compiler {
            reports: self.reports,
            files: Files(checked_files),
            info: CheckedInfo { evaluation_order },
        }
    }
}

pub type File = SimpleFile<String, String>;
pub struct Files<T>(pub Vec<(File, PathBuf, T)>);

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
            Some((file, _, _)) => Ok(file.name()),
            None => Err(files::Error::FileMissing),
        }
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, files::Error> {
        match self.0.get(id) {
            Some((file, _, _)) => Ok(file.source()),
            None => Err(files::Error::FileMissing),
        }
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, files::Error> {
        match self.0.get(id) {
            Some((file, _, _)) => file.line_index((), byte_index),
            None => Err(files::Error::FileMissing),
        }
    }

    fn line_range(
        &'a self,
        id: Self::FileId,
        line_index: usize,
    ) -> Result<std::ops::Range<usize>, files::Error> {
        match self.0.get(id) {
            Some((file, _, _)) => file.line_range((), line_index),
            None => Err(files::Error::FileMissing),
        }
    }
}
