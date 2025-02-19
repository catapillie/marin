use std::collections::{HashMap, HashSet};

use crate::com::{
    ast, ir,
    reporting::{Header, Label, Note, Report},
    Checker,
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn check_have(&mut self, e: &ast::Have) -> ir::Stmt {
        let span = e.span();
        let class_name = e.name.lexeme(self.source);
        let within_label = Label::WithinClassInstantiation(class_name.to_string());

        use super::path::PathQuery as Q;
        let lexeme = ast::Lexeme { span: e.name };
        let class_id = match self.check_var_path(&lexeme) {
            Q::Class(id) => id,
            _ => return ir::Stmt::Nothing,
        };

        self.open_scope(false);

        use ast::Expr as E;
        let mut registered = HashMap::new();
        for item in &e.items {
            let E::Let(item) = item else {
                self.reports.push(
                    Report::error(Header::InvalidExpression())
                        .with_primary_label(Label::Empty, item.span().wrap(self.file))
                        .with_secondary_label(within_label.clone(), span.wrap(self.file))
                        .with_note(Note::InstanceSyntax),
                );
                continue;
            };

            let (_, bindings) = self.check_let_bindings(item);
            for binding in bindings {
                let binding_info = self.get_variable(binding);
                registered.insert(binding_info.name.clone(), binding);
            }
        }

        let info = self.get_class_info(class_id);
        let arity = info.arity;
        let mut instantiated_items = Vec::new();
        let mut missing_items = Vec::new();
        for (i, item) in info.items.iter().enumerate() {
            let Some(binding) = registered.get(&item.name).copied() else {
                missing_items.push(i);
                continue;
            };

            let binding_info = self.get_variable(binding);
            let scheme = binding_info.scheme.clone();
            let expected_scheme = item.scheme.clone();
            instantiated_items.push((expected_scheme, scheme));
        }

        let is_complete = missing_items.is_empty();

        if !is_complete {
            let missing_names = missing_items
                .iter()
                .map(|i| info.items[*i].name.clone())
                .collect();
            self.reports.push(
                Report::error(Header::UninstantiatedItems(class_name.to_string()))
                    .with_primary_label(
                        Label::MissingItems(missing_names, class_name.to_string()),
                        span.wrap(self.file),
                    )
                    .with_secondary_label(Label::ClassDefinition(class_name.to_string()), info.loc),
            );
        }

        self.open_scope(false);

        let current_constraint = ir::Constraint {
            id: class_id,
            class_args: (0..arity.0).map(|_| self.create_fresh_type(None)).collect(),
            associated_args: (0..arity.1).map(|_| self.create_fresh_type(None)).collect(),
        };

        for (wanted_scheme, found_scheme) in instantiated_items {
            let found_type = self.instantiate_scheme(found_scheme);
            let (expected_type, current_contraints) =
                self.instantiate_scheme_keep_constraints(wanted_scheme);
            debug_assert_eq!(current_contraints.len(), 1);

            self.unify(expected_type, found_type, &[]);
            self.unify_constraint(&current_constraint, current_contraints.first().unwrap());
        }

        let mut instantiation_domain = HashSet::new();
        self.collect_constraint_variables(&current_constraint, &mut instantiation_domain);

        let instantiation_constraints = self.solve_constraints();
        for constraint in &instantiation_constraints {
            self.collect_constraint_variables(constraint, &mut instantiation_domain);
        }

        let scheme = ir::InstanceScheme {
            forall: instantiation_domain,
            constraint: current_constraint,
            required_constraints: instantiation_constraints,
        };

        if is_complete {
            println!("{}", self.get_instance_scheme_string(&scheme));
        }

        self.close_scope();
        self.close_scope();

        self.create_entity(ir::Entity::Instance(ir::InstanceInfo {
            loc: span.wrap(self.file),
            scheme,
        }));

        ir::Stmt::Nothing
    }
}
