use super::ir;

pub fn lower(mut modules: Vec<ir::Module>, entities: ir::Entities, dependency_order: Vec<usize>) {
    let mut stmts = Vec::new();
    for file_id in dependency_order {
        let file_stmts = std::mem::take(&mut modules[file_id].stmts);
        stmts.extend_from_slice(&file_stmts);
    }
}
