use super::{checks::CheckModuleOptions, deps};
use crate::com::{
    ir::{self, Entities},
    reporting::Report,
    scope::Scope,
};
use std::collections::{HashMap, HashSet};

pub type Instances = HashSet<ir::InstanceID>;

#[derive(Default)]
pub struct Export<'src> {
    pub was_checked: bool,
    pub exports: HashMap<&'src str, ir::AnyID>,
    pub instances: Vec<ir::InstanceID>,
}

#[derive(Default)]
pub struct ScopeInfo {
    pub name: String,
    pub instances: Instances,
}

pub struct NativeTypes {
    pub int: ir::TypeID,
    pub float: ir::TypeID,
    pub string: ir::TypeID,
    pub bool: ir::TypeID,
}

impl NativeTypes {
    pub fn blank() -> Self {
        Self {
            int: ir::TypeID::whatever(),
            float: ir::TypeID::whatever(),
            string: ir::TypeID::whatever(),
            bool: ir::TypeID::whatever(),
        }
    }
}

pub struct Checker<'src, 'e> {
    pub options: CheckModuleOptions,

    pub source: &'src str,
    pub file: usize,
    pub reports: &'e mut Vec<Report>,

    pub deps: &'e deps::Dependencies,
    pub exports: Vec<Export<'src>>,

    pub scope: Scope<&'src str, ScopeInfo, ir::AnyID>,
    pub label_scope: Scope<&'src str, (), ir::LabelID>,

    pub entities: ir::Entities,
    pub labels: Vec<ir::Label>,
    pub types: Vec<ir::TypeNode>,
    pub native_types: NativeTypes,
    pub publics: HashSet<ir::AnyID>,
    pub current_constraints: Vec<ir::Constraint>,

    generic_counter: usize,
}

impl<'e> Checker<'_, 'e> {
    pub fn new(
        file_count: usize,
        deps: &'e deps::Dependencies,
        reports: &'e mut Vec<Report>,
    ) -> Self {
        let mut checker = Self {
            options: CheckModuleOptions::new(),

            source: "",
            file: 0,
            reports,

            exports: (0..file_count).map(|_| Export::default()).collect(),
            deps,

            scope: Scope::root(),
            label_scope: Scope::root(),
            entities: Entities::default(),
            labels: Vec::new(),
            types: Vec::new(),
            native_types: NativeTypes::blank(),
            publics: HashSet::new(),
            current_constraints: Vec::new(),

            generic_counter: 0,
        };

        // native type bindings
        checker.native_types.int = checker.create_native_type("int", ir::Type::Int);
        checker.native_types.float = checker.create_native_type("float", ir::Type::Float);
        checker.native_types.string = checker.create_native_type("string", ir::Type::String);
        checker.native_types.bool = checker.create_native_type("bool", ir::Type::Bool);

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

    pub fn set_scope_name(&mut self, name: String) {
        self.scope.infos_mut().name = name;
    }

    pub fn build_scope_name(&self) -> String {
        let mut names = self
            .scope
            .infos_iter()
            .map(|info| info.name.to_string())
            .collect::<Vec<_>>();
        names.reverse();
        names.join(".")
    }

    pub fn get_generic_unique_id(&mut self) -> usize {
        self.generic_counter += 1;
        self.generic_counter
    }
}

macro_rules! checker_print {
    ($self:ident, $($arg:tt)*) => {
        if $self.options.is_verbose {
            eprintln!($($arg)*)
        }
    };
}

pub(crate) use checker_print;
