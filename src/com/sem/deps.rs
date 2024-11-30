use petgraph::{algo, prelude::DiGraphMap};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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

        let mut import_spans = HashMap::<_, Span>::new();
        for (query, span) in queries {
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
            } else if let Some(first_span) = import_spans.get(&dep_id) {
                reports.push(
                    Report::warning(Header::FileReimported(short_dep_path.display().to_string()))
                        .with_primary_label(Label::Empty, span.wrap(file_id))
                        .with_secondary_label(
                            Label::FirstImportHere(short_dep_path.display().to_string()),
                            first_span.wrap(file_id),
                        ),
                );
            }

            import_spans.entry(dep_id).or_insert(span);
            graph.add_edge(file_id, dep_id, ());
        }
    }

    graph
}

fn navigate_query(from: impl AsRef<Path>, query: &Query) -> Option<PathBuf> {
    let mut path = from.as_ref().to_path_buf();
    for part in &query.0 {
        match part {
            Part::Dir(name, _) => path.push(name),
            Part::Super(_) => {
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
    Dir(String, Span),
    Super(Span),
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
