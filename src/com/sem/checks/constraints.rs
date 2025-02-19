use crate::com::{ir, Checker};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn require_class_constraint(&mut self, constraint: ir::Constraint) {
        self.current_constraints.push(constraint);
    }

    pub fn take_constraint_context(&mut self) -> Vec<ir::Constraint> {
        std::mem::take(&mut self.current_constraints)
    }

    pub fn restore_constraint_context(&mut self, constraints: Vec<ir::Constraint>) {
        self.current_constraints = constraints;
    }

    pub fn solve_constraints(&mut self) -> Vec<ir::Constraint> {
        let current_constraints = self.take_constraint_context();
        let mut remaining = Vec::new();
        let mut relevant = Vec::new();
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
            // living deeper than were the contraints are being solved
            if self.is_relevant_constraint(&constraint) {
                relevant.push(constraint);
                continue;
            }

            // otherwise, fort about it
            // it'll be solved later
            remaining.push(constraint);
        }

        for constraint in concrete {
            self.check_constraint(constraint);
        }

        self.restore_constraint_context(remaining);
        relevant
    }

    fn get_known_instances(&self) -> Vec<&ir::InstanceInfo> {
        self.scope
            .infos()
            .flatten()
            .map(|id| self.get_instance_info(*id))
            .collect::<Vec<_>>()
    }

    fn check_constraint(&mut self, constraint: ir::Constraint) {
        println!("    * check {}", self.get_constraint_string(&constraint));
        // for instance in self.get_known_instances() {
        //     println!("      ? -> {}", self.get_instance_scheme_string(&instance.scheme));
        // }
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
