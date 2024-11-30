use petgraph::{algo, prelude::DiGraphMap};
use std::path::{Path, PathBuf};

use crate::com::{
    ast,
    compiler::{Files, Parsed},
    file_tree::FileTree,
    loc::Span,
    reporting::{Header, Label, Note, Report},
};

pub type DepGraph = DiGraphMap<usize, ()>;

pub fn build_dependency_graph(files: &Files<Parsed>, reports: &mut Vec<Report>) -> DepGraph {
    let mut graph = DepGraph::with_capacity(files.0.len(), 0);

    let mut file_tree = FileTree::new();
    for (file_id, (_, path, _)) in files.0.iter().enumerate() {
        file_tree.add_file(path, file_id);
    }

    let current_dir = std::env::current_dir()
        .expect("need access to current directory")
        .canonicalize()
        .expect("couldn't normalize current directory");

    for (file_id, (file, path, Parsed(ast::File(ast)))) in files.0.iter().enumerate() {
        graph.add_edge(file_id, file_id, ());

        let source = file.source();
        let file_name = file.name();

        let mut queries = Vec::new();
        for expr in ast {
            for item in ast::preorder_traversal(expr) {
                process_import(item, file_id, source, reports, &mut queries);
            }
        }

        for (query, span) in queries {
            if matches!(query.0.last(), Some(Part::Super)) {
                reports.push(
                    Report::error(Header::InvalidDependencyPath())
                        .with_primary_label(Label::TrailingSuper, span.wrap(file_id)),
                );
                continue;
            }

            let Some(full_dep_path) = navigate_query(path.parent().unwrap(), &query) else {
                reports.push(
                    Report::error(Header::InvalidDependencyPath())
                        .with_primary_label(Label::Empty, span.wrap(file_id)),
                );
                continue;
            };

            let short_dep_path = match full_dep_path.strip_prefix(&current_dir) {
                Ok(short) => short,
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
                    true => Report::error(Header::UnstagedDependency(
                        short_dep_path.display().to_string(),
                    ))
                    .with_note(Note::ConsiderStage(short_dep_path.display().to_string())),
                    false => Report::error(Header::NoSuchDependency(
                        short_dep_path.display().to_string(),
                    )),
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
            }

            graph.add_edge(file_id, dep_id, ());
        }
    }

    graph
}

fn navigate_query(from: impl AsRef<Path>, query: &Query) -> Option<PathBuf> {
    let mut path = from.as_ref().to_path_buf();
    for part in &query.0 {
        match part {
            Part::Dir(name) => path.push(name),
            Part::Super => {
                if !path.pop() {
                    return None;
                }
            }
        }
    }
    path.set_extension("mar");
    Some(path)
}

#[derive(Debug)]
enum Part {
    Dir(String),
    Super,
}

#[derive(Debug)]
struct Query(pub Vec<Part>);

fn process_import(
    expr: &ast::Expr,
    file_id: usize,
    source: &str,
    reports: &mut Vec<Report>,
    queries: &mut Vec<(Query, Span)>,
) {
    let ast::Expr::Import(import) = expr else {
        return;
    };

    for query in &import.queries {
        let Some(parts) = process_import_query(query, source) else {
            reports.push(
                Report::error(Header::InvalidImportQuery())
                    .with_primary_label(Label::Empty, expr.span().wrap(file_id)),
            );
            return;
        };

        queries.push((parts, query.span()));
    }
}

fn process_import_query(expr: &ast::Expr, source: &str) -> Option<Query> {
    match expr {
        ast::Expr::Var(lex) => {
            let lexeme = lex.span.lexeme(source).to_string();
            Some(Query(vec![Part::Dir(lexeme)]))
        }
        ast::Expr::Super(..) => Some(Query(vec![Part::Super])),
        ast::Expr::Access(access) => {
            let mut q = process_import_query(&access.accessed, source)?;
            q.0.push(Part::Dir(access.name.lexeme(source).to_string()));
            Some(q)
        }
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
