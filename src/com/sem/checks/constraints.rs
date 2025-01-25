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
}
