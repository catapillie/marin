use super::deps;
use crate::com::{
    ir::{self},
    reporting::Report,
    scope::Scope,
};

pub type Instances = Vec<ir::EntityID>;

pub struct Checker<'src, 'e> {
    pub source: &'src str,
    pub file: usize,
    pub deps: &'e deps::DepGraph,
    pub reports: &'e mut Vec<Report>,

    pub scope: Scope<&'src str, Instances, ir::EntityID>,
    pub label_scope: Scope<&'src str, (), ir::LabelID>,

    pub entities: Vec<ir::Entity>,
    pub labels: Vec<ir::Label>,
    pub types: Vec<ir::TypeNode>,

    pub current_constraints: Vec<ir::Constraint>,
}

impl<'src, 'e> Checker<'src, 'e> {
    pub fn new(
        source: &'src str,
        file: usize,
        deps: &'e deps::DepGraph,
        reports: &'e mut Vec<Report>,
    ) -> Self {
        let mut checker = Self {
            source,
            file,
            deps,
            reports,

            scope: Scope::root(),
            label_scope: Scope::root(),

            entities: Vec::new(),
            labels: Vec::new(),
            types: Vec::new(),

            current_constraints: Vec::new(),
        };

        // native type bindings
        checker.create_native_type("int", ir::Type::Int);
        checker.create_native_type("float", ir::Type::Float);
        checker.create_native_type("string", ir::Type::String);
        checker.create_native_type("bool", ir::Type::Bool);

        checker
    }

    pub fn open_scope(&mut self, blocking: bool) {
        self.scope.open(false);
        self.label_scope.open(blocking);
    }

    pub fn close_scope(&mut self) {
        self.scope.close();
        self.label_scope.close();
    }
}
