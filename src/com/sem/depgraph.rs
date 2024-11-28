use petgraph::Graph;
use std::{collections::HashMap, path::Path};

use crate::com::{
    ast,
    compiler::{Files, Parsed},
    reporting::{Header, Label, Report},
};

pub type DepGraph = Graph<usize, ()>;

pub fn build_dependency_graph(files: &Files<Parsed>, reports: &mut Vec<Report>) -> DepGraph {
    let mut graph = Graph::new();

    let mut files_by_path = HashMap::new();
    let indices = files
        .0
        .iter()
        .enumerate()
        .map(|(i, (f, _))| {
            files_by_path.insert(f.name(), i);
            graph.add_node(i)
        })
        .collect::<Vec<_>>();

    for (id, (f, Parsed(ast::File(ast)))) in files.0.iter().enumerate() {
        println!("--> [{id}] {}:", f.name());

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
                        println!("  --> [{dep_id}] {dep_path_display}");
                        graph.add_edge(indices[id], indices[dep_id], ());
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
