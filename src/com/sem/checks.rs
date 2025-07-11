mod constraints;
mod entities;
mod labels;
mod patterns;
mod signatures;
mod variables;

mod stmt;
mod stmt_alias;
mod stmt_class;
mod stmt_have;
mod stmt_import;
mod stmt_let;
mod stmt_record;
mod stmt_union;

mod expr;
mod expr_access;
mod expr_array;
mod expr_block;
mod expr_break;
mod expr_builtin;
mod expr_call;
mod expr_conditional;
mod expr_fun;
mod expr_index;
mod expr_literal;
mod expr_ops;
mod expr_record;
mod expr_skip;
mod expr_tuple;
mod expr_var;

mod branch;
mod branch_else;
mod branch_if;
mod branch_loop;
mod branch_match;
mod branch_while;

mod type_call;
mod type_fun;
mod type_tuple;
mod type_var;
mod types;

mod path;
mod path_access;
mod path_access_class;
mod path_access_import;
mod path_access_record;
mod path_access_union;
mod path_call;
mod path_var;

mod file;
pub use file::CheckModuleOptions;
