use std::collections::{BTreeSet, HashMap};

use crate::com::{
    Checker, ast, ir,
    reporting::{Header, Label, Note, Report},
    sem::checker::checker_print,
};

impl Checker<'_, '_> {
    pub fn check_have(&mut self, e: &ast::Have, public: bool) -> ir::Stmt {
        let span = e.span();

        use ir::PathQuery as Q;
        let class_id = match self.check_path_or_type(&e.class) {
            Q::Class(id) => id,
            Q::Missing => return ir::Stmt::Nothing,
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidClass())
                        .with_primary_label(Label::NotAClass, span.wrap(self.file)),
                );
                return ir::Stmt::Nothing;
            }
        };

        let info = self.entities.get_class_info(class_id);
        let class_name = info.name.clone();
        let within_label = Label::WithinClassInstantiation(class_name.to_string());

        self.open_scope(false);

        use ast::Expr as E;
        let mut registered = HashMap::new();
        let mut stmts = Vec::new();
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

            let (stmt, bindings) = self.check_let_bindings(item, false);
            stmts.push(stmt);
            for binding in bindings {
                let binding_info = self.entities.get_variable_info(binding);
                registered.insert(binding_info.name.clone(), binding);
            }
        }

        let info = self.entities.get_class_info(class_id);
        let arity = info.arity;
        let mut instantiated_items = Vec::new();
        let mut missing_items = Vec::new();
        let mut item_infos = Vec::new();
        for (i, item) in info.items.iter().enumerate() {
            let Some(binding) = registered.get(&item.name).copied() else {
                missing_items.push(i);
                continue;
            };

            let binding_info = self.entities.get_variable_info(binding);
            let is_concrete = binding_info.scheme.constraints.is_empty();
            let scheme = binding_info.scheme.clone();
            let expected_scheme = item.scheme.clone();
            instantiated_items.push((expected_scheme, scheme));
            item_infos.push(ir::InstanceItemInfo {
                binding,
                is_concrete,
            });
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
            loc: e.class.span().wrap(self.file),
            class_args: (0..arity.0).map(|_| self.create_fresh_type(None)).collect(),
            associated_args: (0..arity.1).map(|_| self.create_fresh_type(None)).collect(),
            constraint_trace: ir::ConstraintTrace::default(),
        };

        for (wanted_scheme, found_scheme) in instantiated_items {
            let found_type = self.instantiate_scheme_same_constraint_trace(
                found_scheme,
                Some(e.span().wrap(self.file)),
            );
            let (expected_type, current_constraints) =
                self.instantiate_scheme_keep_constraints(wanted_scheme);
            debug_assert_eq!(current_constraints.len(), 1);

            self.unify(expected_type, found_type, &[]);
            self.unify_constraint(&current_constraint, current_constraints.first().unwrap());
        }

        let mut instantiation_domain = BTreeSet::new();
        self.collect_constraint_variables(&current_constraint, &mut instantiation_domain);

        let (_, instantiation_constraints) = self.solve_constraints();
        for constraint in &instantiation_constraints {
            self.collect_constraint_variables(constraint, &mut instantiation_domain);
        }

        let scheme = ir::InstanceScheme {
            forall: instantiation_domain,
            constraint: current_constraint,
            required_constraints: instantiation_constraints,
        };

        if is_complete {
            checker_print!(self, "{}", self.get_instance_scheme_string(&scheme));
        }

        self.close_scope();
        self.close_scope();

        if is_complete {
            let instance_id = self.entities.next_instance_id();
            self.entities.create_instance(ir::InstanceInfo {
                loc: span.wrap(self.file),
                scheme,
                original: instance_id,
                items: item_infos.into(),
            });
            self.scope.infos_mut().instances.insert(instance_id);
            self.set_entity_public(instance_id.wrap(), public);

            return ir::Stmt::Have {
                stmts: stmts.into(),
            };
        }

        // fail
        ir::Stmt::Nothing
    }
}
