use crate::com::{
    Checker, ir,
    reporting::{Header, Label, Report},
};

impl Checker<'_, '_> {
    pub fn require_class_constraint(&mut self, constraint: ir::Constraint) {
        self.current_constraints.push(constraint);
    }

    pub fn take_constraint_context(&mut self) -> Vec<ir::Constraint> {
        std::mem::take(&mut self.current_constraints)
    }

    pub fn restore_constraint_context(&mut self, constraints: Vec<ir::Constraint>) {
        self.current_constraints = constraints;
    }

    pub fn solve_constraints(&mut self) -> (Vec<ir::Solution>, Vec<ir::Constraint>) {
        let mut current_constraints = self.take_constraint_context();
        let mut irrelevant = Vec::new();
        let mut solutions = Vec::new();

        loop {
            let mut partial = Vec::new();
            let mut concrete = Vec::new();

            for constraint in current_constraints {
                // if the constraint contains no variables
                // then we say it is "concrete"
                // and must thus be checked
                if self.is_concrete_constraint(&constraint) {
                    concrete.push(constraint);
                    continue;
                }

                // if not, it might still be relevant
                // which means intuitively that it constrains at least one type variable
                // living deeper than were the constraints are being solved
                if self.is_relevant_constraint(&constraint) {
                    partial.push(constraint);
                    continue;
                }

                // otherwise, forget about it
                // it'll be solved later
                irrelevant.push(constraint);
            }

            // partially concrete constraints might become concrete after checking
            // constraints which are already concrete
            // so we put them back in the current constraint list
            current_constraints = partial;

            // if this iteration of solving did not yield any new concrete constraints
            // then we are done (otherwise we would loop forever)
            if concrete.is_empty() {
                break;
            }

            // each concrete constraint is checked to have a matching instance
            // which may generate additional constraints
            // so we need to decide whether to continue the loop or not
            for mut constraint in concrete {
                let trace = std::mem::take(&mut constraint.constraint_trace);
                if let Some((instance_id, mut additional, additional_constraint_id)) =
                    self.check_constraint(constraint)
                {
                    solutions.push(ir::Solution {
                        trace,
                        instance_id,
                        additional_constraint_id,
                    });
                    current_constraints.append(&mut additional);
                }
            }
        }

        self.restore_constraint_context(irrelevant);
        (solutions, current_constraints)
    }

    fn check_constraint(
        &mut self,
        constraint: ir::Constraint,
    ) -> Option<(ir::InstanceID, Vec<ir::Constraint>, usize)> {
        // get available instances
        let mut matching_instances = Vec::new();
        for (instance_id, instance) in self.get_known_instances() {
            // only keep potentially relevant constraints
            let scheme = &instance.scheme;
            if scheme.constraint.id != constraint.id {
                continue;
            }

            // instantiate the instance (in case it's polymorphic or uses other constraints)
            let (additional_constraint_id, instance_constraint, additional) =
                self.instantiate_instance_scheme(scheme.clone());

            // attempt to unify, if yes then we have a matching instance!!!!!!!!!!!!!!!
            if self.try_unify_constraint_args(&constraint, &instance_constraint) {
                matching_instances.push((
                    instance_id,
                    instance_constraint,
                    additional,
                    additional_constraint_id,
                ));
            }
        }

        // ensure we have exactly one instance
        let constr_string = self.get_constraint_string(&constraint);
        if matching_instances.is_empty() {
            self.reports.push(
                Report::error(Header::UnsatisfiedConstraint(constr_string.clone()))
                    .with_primary_label(
                        Label::ConstraintOrigin(constr_string.clone()),
                        constraint.loc,
                    ),
            );
            return None;
        } else if matching_instances.len() > 1 {
            let id_a = matching_instances[0].0;
            let loc_a = self.entities.get_instance_info(id_a).loc;
            let id_b = matching_instances[1].0;
            let loc_b = self.entities.get_instance_info(id_b).loc;
            self.reports.push(
                Report::error(Header::AmbiguousConstraintSolution(constr_string.clone()))
                    .with_primary_label(
                        Label::MatchingInstances(constr_string.clone()),
                        constraint.loc,
                    )
                    .with_secondary_label(Label::SuchInstance, loc_a)
                    .with_secondary_label(Label::SuchInstance, loc_b),
            );
            return None;
        }

        // retrieve the matching instance
        // unify the associated type arguments

        let (instance_id, matching_constraint, additional_constraints, additional_constraint_id) =
            matching_instances.pop().unwrap();
        self.unify_constraint(&constraint, &matching_constraint);

        Some((
            instance_id,
            additional_constraints,
            additional_constraint_id,
        ))
    }

    fn get_known_instances(&self) -> Vec<(ir::InstanceID, ir::InstanceInfo)> {
        let mut in_scope = self
            .scope
            .infos_iter()
            .flat_map(|info| &info.instances)
            .map(|id| (*id, self.entities.get_instance_info(*id).clone()))
            .collect::<Vec<_>>();
        in_scope.sort_by_key(|(_, info)| info.original.0);
        in_scope.dedup_by_key(|(_, info)| info.original.0);
        in_scope
    }

    fn is_concrete_constraint(&mut self, constraint: &ir::Constraint) -> bool {
        constraint
            .class_args
            .iter()
            .all(|arg| self.is_concrete_type(*arg))
    }

    fn is_relevant_constraint(&mut self, constraint: &ir::Constraint) -> bool {
        constraint
            .class_args
            .iter()
            .any(|arg| self.is_relevant_type(*arg))
    }
}
