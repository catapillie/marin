use petgraph::{algo, prelude::DiGraphMap};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::com::{
    ast,
    compiler::{Files, Parsed},
    file_tree::FileTree,
    loc::Span,
    reporting::{Header, Label, Note, Report},
};

pub fn get_marin_path_builtin(dir: &str) -> PathBuf {
    std::env::current_exe()
        .expect("need access to current executable directory")
        .canonicalize()
        .expect("couldn't normalize current executable directory")
        .parent()
        .unwrap()
        .join(dir)
}

pub fn get_marin_std_path() -> PathBuf {
    get_marin_path_builtin("std")
}

pub struct Dependencies {
    pub graph: DepGraph,
    pub info: DepInfo,
}

pub type QueryUIDs = HashSet<usize>;
pub type DepGraph = DiGraphMap<usize, QueryUIDs>;

#[derive(Default)]
pub struct DepInfo {
    pub prelude_file: Option<usize>,
}

pub fn analyse_dependencies(
    files: &Files<Parsed>,
    is_std_staged: bool,
    reports: &mut Vec<Report>,
) -> Dependencies {
    let mut file_tree = FileTree::new();
    for (file_id, (_, path, _)) in files.0.iter().enumerate() {
        file_tree.add_file(path, file_id);
    }

    // initialize output
    let mut graph = DepGraph::with_capacity(files.0.len(), 0);
    let mut info = DepInfo::default();

    // use prelude file
    if is_std_staged {
        let prelude_path = get_marin_path_builtin("std").join("prelude.mar");
        let prelude_file = file_tree.get_by_path(&prelude_path).copied();
        info.prelude_file = prelude_file;
    }

    let current_dir = std::env::current_dir()
        .expect("need access to current directory")
        .canonicalize()
        .expect("couldn't normalize current directory");

    for (file_id, (file, path, Parsed(ast::File(ast), file_info))) in files.0.iter().enumerate() {
        graph.add_edge(file_id, file_id, Default::default());

        if is_std_staged && !file_info.is_from_std {
            if let Some(prelude_id) = info.prelude_file {
                graph.add_edge(file_id, prelude_id, Default::default());
            }
        }

        let source = file.source();
        let file_name = file.name();

        let mut queries = Vec::new();
        for expr in ast {
            for item in ast::preorder_traversal(expr) {
                process_import(item, file_id, source, reports, &mut queries);
            }
        }

        let mut import_spans = HashMap::<_, Span>::new();
        for (query, is_total, uid, span) in queries {
            if query.0.len() > 2 {
                for i in 1..(query.0.len() - 1) {
                    let curr = &query.0[i];
                    let prev = &query.0[i - 1];

                    match (prev, curr) {
                        (Part::Dir(_, prev_span), Part::Super(curr_span)) => {
                            let loc = Span::combine(*prev_span, *curr_span).wrap(file_id);
                            reports.push(
                                Report::warning(Header::RedundantSuper())
                                    .with_primary_label(Label::RedundantImportPath, loc),
                            );
                        }
                        _ => continue,
                    }
                }
            }

            if matches!(query.0.last(), Some(Part::Super(_))) {
                reports.push(
                    Report::error(Header::InvalidDependencyPath())
                        .with_primary_label(Label::TrailingSuper, span.wrap(file_id)),
                );
                continue;
            }

            let Some((full_dep_path, used_builtin)) =
                navigate_query(path.parent().unwrap(), &query)
            else {
                reports.push(
                    Report::error(Header::InvalidDependencyPath())
                        .with_primary_label(Label::Empty, span.wrap(file_id)),
                );
                continue;
            };

            let short_dep_path = match full_dep_path.strip_prefix(&current_dir) {
                _ if used_builtin => get_query_string(&query),
                Ok(short) => short.display().to_string(),
                Err(_) => {
                    reports.push(
                        Report::error(Header::OutsideDependency())
                            .with_primary_label(Label::Empty, span.wrap(file_id)),
                    );
                    continue;
                }
            };

            let Some(&dep_id) = file_tree.get_by_path(&full_dep_path) else {
                let rep = match full_dep_path.exists() && !full_dep_path.is_dir() {
                    true => Report::error(Header::UnstagedDependency(short_dep_path.clone()))
                        .with_note(Note::ConsiderStage(short_dep_path.clone())),
                    false => Report::error(Header::NoSuchDependency(short_dep_path.clone())),
                };
                reports.push(rep.with_primary_label(
                    Label::ImportedInFile(file_name.clone()),
                    span.wrap(file_id),
                ));
                continue;
            };

            if file_id == dep_id {
                reports.push(
                    Report::warning(Header::SelfDependency(file_name.clone())).with_primary_label(
                        Label::ImportedInItself(file_name.clone()),
                        span.wrap(file_id),
                    ),
                );
                continue;
            } else if let Some(first_span) = import_spans.get(&dep_id) {
                // no need to warning if this is only a partial reimport
                // for example, it's okay to reimport in order to set aliases
                if is_total {
                    reports.push(
                        Report::warning(Header::FileReimported(short_dep_path.clone()))
                            .with_primary_label(Label::Empty, span.wrap(file_id))
                            .with_secondary_label(
                                Label::FirstImportHere(short_dep_path.clone()),
                                first_span.wrap(file_id),
                            ),
                    );
                }
            }

            if is_total {
                import_spans.entry(dep_id).or_insert(span);
            }

            match graph.edge_weight_mut(file_id, dep_id) {
                Some(uids) => {
                    uids.insert(uid);
                }
                None => {
                    let mut uids = HashSet::new();
                    uids.insert(uid);
                    graph.add_edge(file_id, dep_id, uids);
                }
            };
        }
    }

    Dependencies { graph, info }
}

fn navigate_query(from: impl AsRef<Path>, query: &Query) -> Option<(PathBuf, bool)> {
    let mut path = from.as_ref().to_path_buf();
    let mut is_first = true;
    let mut used_builtin = false;
    for part in &query.0 {
        match part {
            Part::Dir(name, _) => path.push(name),
            Part::Super(_) => {
                if !path.pop() {
                    return None;
                }
            }
            Part::Builtin(name) => {
                if !is_first {
                    return None;
                } else {
                    used_builtin = true;
                    path = get_marin_path_builtin(name);
                }
            }
        }
        is_first = false;
    }
    path.set_extension("mar");
    Some((path, used_builtin))
}

fn get_query_string(query: &Query) -> String {
    let part_strings = query
        .0
        .iter()
        .map(|part| match part {
            Part::Dir(name, _) => name.clone(),
            Part::Builtin(name) => format!("\"{}\"", name),
            Part::Super(_) => "super".to_string(),
        })
        .collect::<Vec<_>>();
    part_strings.join(".")
}

#[derive(Debug)]
enum Part {
    Dir(String, Span),
    Builtin(String),
    Super(Span),
}

#[derive(Debug)]
struct Query(pub Vec<Part>);

fn process_import(
    expr: &ast::Expr,
    file_id: usize,
    source: &str,
    reports: &mut Vec<Report>,
    queries: &mut Vec<(Query, bool, usize, Span)>,
) {
    match expr {
        ast::Expr::Import(import) => {
            for query in &import.queries {
                let Some(parts) = process_import_query(&query.query, source) else {
                    reports.push(
                        Report::error(Header::InvalidImportQuery())
                            .with_primary_label(Label::Empty, expr.span().wrap(file_id)),
                    );
                    return;
                };

                queries.push((parts, true, query.uid, query.query.span()));
            }
        }
        ast::Expr::ImportFrom(import) => {
            let Some(parts) = process_import_query(&import.path_query, source) else {
                reports.push(
                    Report::error(Header::InvalidImportQuery())
                        .with_primary_label(Label::Empty, expr.span().wrap(file_id)),
                );
                return;
            };

            queries.push((
                parts,
                false,
                import.path_query_uid,
                import.path_query.span(),
            ));
        }
        _ => {}
    }
}

fn process_import_query(expr: &ast::Expr, source: &str) -> Option<Query> {
    match expr {
        ast::Expr::Access(access) => {
            let mut q = process_import_query(&access.accessed, source)?;
            q.0.push(process_query_accessor(&access.accessor, source)?);
            Some(q)
        }
        _ => Some(Query(vec![process_query_accessor(expr, source)?])),
    }
}

fn process_query_accessor(expr: &ast::Expr, source: &str) -> Option<Part> {
    match expr {
        ast::Expr::Var(lex) => Some(Part::Dir(lex.span.lexeme(source).to_string(), lex.span)),
        ast::Expr::Super(lex) => Some(Part::Super(lex.span)),
        ast::Expr::String(lex) => match &source[lex.span.start + 1..lex.span.end - 1] {
            "" => None,
            name => Some(Part::Builtin(name.to_string())),
        },
        _ => None,
    }
}

pub fn sort_dependencies<T>(
    graph: &DepGraph,
    files: &Files<T>,
    reports: &mut Vec<Report>,
) -> Vec<Vec<usize>> {
    let post = algo::tarjan_scc(graph);
    for scc in &post {
        if scc.len() > 1 {
            reports.push(Report::error(Header::DependencyCycle()).with_note(
                Note::CyclicDependencies(
                    scc.iter().map(|&i| files.0[i].0.name().clone()).collect(),
                ),
            ));
        }
    }
    post
}
