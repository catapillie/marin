use super::{
    Checker, Parser, ast, emit,
    ir::{self},
    low,
    reporting::{Header, Report},
    sem::{self},
};
use codespan_reporting::{
    files::{self, SimpleFile},
    term::{
        self, Config,
        termcolor::{ColorChoice, StandardStream},
    },
};
use std::path::{Path, PathBuf};

pub struct Compiler<Stage, Info> {
    pub(crate) reports: Vec<Report>,
    files: Files<Stage>,
    info: Info,
}

// compiler stage
#[derive(Default, Clone)]
pub struct StagedFileInfo {
    pub is_from_std: bool,
}
impl StagedFileInfo {
    pub fn marin_std_file() -> Self {
        Self { is_from_std: true }
    }
}
pub enum Staged {
    File {
        canonicalized: PathBuf,
        staged_path: PathBuf,
        info: StagedFileInfo,
    },
    Source(String, String),
}

pub struct Sourced(Option<PathBuf>, StagedFileInfo);

pub struct Parsed(pub Option<PathBuf>, pub ast::File, pub StagedFileInfo);

pub struct Checked(pub ir::Module);

pub struct Compiled;

// compiler info
pub struct StagedInfo {
    is_std_staged: bool,
}

pub struct SourceInfo {
    is_std_staged: bool,
}

pub struct ParsedInfo {
    is_std_staged: bool,
}

pub struct CheckedInfo {
    entities: ir::Entities,
    dependency_order: Vec<usize>,
}

pub struct CompiledInfo {
    pub bytecode: Vec<u8>,
}

// compiler initialization
pub fn init() -> Compiler<Staged, StagedInfo> {
    Compiler {
        reports: Vec::new(),
        files: Files::default(),
        info: StagedInfo {
            is_std_staged: false,
        },
    }
}

impl<T, I> Compiler<T, I> {
    pub fn into_content(self) -> I {
        self.info
    }

    pub fn is_fatal(&self) -> bool {
        self.reports.iter().any(Report::is_fatal)
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
            Staged::File {
                canonicalized: full_path,
                staged_path: path.to_path_buf(),
                info,
            },
        ))
    }

    #[allow(dead_code)]
    pub fn add_source(&mut self, name: impl ToString, contents: impl ToString) {
        self.files.0.push((
            File::new(String::new(), String::new()),
            Staged::Source(name.to_string(), contents.to_string()),
        ))
    }

    pub fn add_marin_std(&mut self) {
        if self.info.is_std_staged {
            panic!("marin std library is already staged");
        }

        let marin_std_path = sem::get_marin_std_path();
        self.add_dir_with_info(&marin_std_path, StagedFileInfo::marin_std_file());
        self.info.is_std_staged = true;
    }

    fn read_file(staged: Staged, reports: &mut Vec<Report>) -> File {
        match staged {
            Staged::File {
                canonicalized: _,
                staged_path: path,
                info: _,
            } => {
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
            Staged::Source(name, source) => File::new(name, source),
        }
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
            .map(|(_, f)| {
                let (opt_path, staged_info) = match &f {
                    Staged::File {
                        canonicalized: path_buf,
                        staged_path: _,
                        info,
                    } => (Some(path_buf.clone()), info.clone()),
                    Staged::Source(_, _) => (None, StagedFileInfo::default()),
                };

                (
                    Self::read_file(f, &mut reports),
                    Sourced(opt_path, staged_info),
                )
            })
            .collect();
        self.reports.append(&mut reports);

        Compiler {
            reports: self.reports,
            files: Files(source_files),
            info: SourceInfo {
                is_std_staged: self.info.is_std_staged,
            },
        }
    }
}

impl Compiler<Sourced, SourceInfo> {
    fn parse_file(
        opt_path: Option<PathBuf>,
        file: &File,
        id: usize,
        info: StagedFileInfo,
        reports: &mut Vec<Report>,
    ) -> Parsed {
        let mut parser = Parser::new(file.source(), id, reports);
        Parsed(opt_path, parser.parse_file(), info)
    }

    pub fn parse(mut self) -> Compiler<Parsed, ParsedInfo> {
        let mut reports = Vec::new();
        let parsed_files = self
            .files
            .0
            .into_iter()
            .enumerate()
            .map(|(id, (f, Sourced(opt_path, info)))| {
                let parsed = Self::parse_file(opt_path, &f, id, info, &mut reports);
                (f, parsed)
            })
            .collect();
        self.reports.append(&mut reports);

        Compiler {
            reports: self.reports,
            files: Files(parsed_files),
            info: ParsedInfo {
                is_std_staged: self.info.is_std_staged,
            },
        }
    }
}

impl Compiler<Parsed, ParsedInfo> {
    pub fn check(mut self) -> Compiler<Checked, CheckedInfo> {
        let deps =
            sem::analyse_dependencies(&self.files, self.info.is_std_staged, &mut self.reports);
        let order = sem::sort_dependencies(&deps.graph, &self.files, &mut self.reports);

        let files = self.files.0;
        let mut irs = Vec::new();
        irs.resize_with(files.len(), || None);

        let mut reports = Vec::new();
        let mut checker = Checker::new(files.len(), &deps, &mut reports);

        for scc in &order {
            for id in scc {
                let id = *id;
                let (file, Parsed(_, ast, info)) = &files[id];

                let options = sem::CheckModuleOptions::new()
                    .set_verbose(!info.is_from_std)
                    .set_import_prelude(self.info.is_std_staged);
                let ir = checker.check_module(file.name(), id, file.source(), ast, options);
                irs[id] = Some(Checked(ir))
            }
        }

        let dependency_order = order.into_iter().flatten().collect();

        let entities = checker.entities;
        self.reports.append(&mut reports);

        let checked_files = files
            .into_iter()
            .zip(irs)
            .map(|((file, _), ir)| {
                let ir = ir.unwrap_or_else(|| panic!("file '{}' was left unchecked", file.name()));
                (file, ir)
            })
            .collect();

        Compiler {
            reports: self.reports,
            files: Files(checked_files),
            info: CheckedInfo {
                entities,
                dependency_order,
            },
        }
    }
}

impl Compiler<Checked, CheckedInfo> {
    pub fn emit(self) -> Compiler<Compiled, CompiledInfo> {
        let mut modules = Vec::with_capacity(self.files.0.len());
        let mut compiled_files = Vec::with_capacity(self.files.0.len());
        for (file, Checked(module)) in self.files.0 {
            modules.push(module);
            compiled_files.push((file, Compiled));
        }

        let lowered = low::lower(modules, self.info.entities, self.info.dependency_order);
        let bytecode = emit::emit(lowered).expect("failed to generate bytecode");

        Compiler {
            reports: self.reports,
            files: Files(compiled_files),
            info: CompiledInfo { bytecode },
        }
    }
}

pub type File = SimpleFile<String, String>;
pub struct Files<T>(pub Vec<(File, T)>);

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
