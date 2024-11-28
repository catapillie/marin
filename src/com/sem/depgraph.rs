use petgraph::{algo, prelude::DiGraphMap};
use std::{collections::HashMap, path::Path};

use crate::com::{
    ast,
    compiler::{Files, Parsed},
    reporting::{Header, Label, Report},
};

pub type DepGraph = DiGraphMap<usize, ()>;

pub fn build_dependency_graph(files: &Files<Parsed>, reports: &mut Vec<Report>) -> DepGraph {
    let mut graph = DepGraph::with_capacity(files.0.len(), 0);
    let files_by_path = files
        .0
        .iter()
        .enumerate()
        .map(|(i, (f, _))| (f.name(), i))
        .collect::<HashMap<_, _>>();

    for (id, (f, Parsed(ast::File(ast)))) in files.0.iter().enumerate() {
        let source = f.source();
        let path = Path::new(f.name());
        for expr in ast {
            let exprs = ast::preorder_traversal(expr);
            for expr in exprs {
                let import = match expr {
                    ast::Expr::Import(import) => import,
                    _ => continue,
                };

                for query in &import.queries {
                    let q = match query {
                        ast::Expr::String(lex) => {
                            let lit = lex.span.lexeme(source);
                            &lit[1..lit.len() - 1]
                        }
                        _ => continue,
                    };

                    let relative_path = Path::new(&q);
                    let Some(parent) = path.parent() else {
                        continue;
                    };

                    let dep_path_display = format!(
                        "{}.mar",
                        parent.join(relative_path).to_string_lossy().into_owned()
                    );
                    let dep_path = Path::new(&dep_path_display);

                    if let Some(&dep_id) = files_by_path.get(&dep_path_display) {
                        graph.add_edge(id, dep_id, ());
                    } else {
                        let rep = match dep_path.exists() {
                            true => Report::error(Header::UnstagedDependency(dep_path_display)),
                            false => Report::error(Header::NoSuchDependency(dep_path_display)),
                        }
                        .with_secondary_label(
                            Label::ImportedInFile(path.display().to_string()),
                            query.span().wrap(id),
                        );
                        reports.push(rep);
                    }
                }
            }
        }
    }

    graph
}

pub fn sort_dependencies<T>(
    graph: &DepGraph,
    files: &Files<T>,
    reports: &mut Vec<Report>,
) -> Vec<Vec<usize>> {
    let post = algo::tarjan_scc(graph);
    for scc in &post {
        println!("->");
        for &id in scc.iter() {
            println!("  -> [{id}] {}", files.0[id].0.name());
        }
    }
    post
}
