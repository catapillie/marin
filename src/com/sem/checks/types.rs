use std::collections::{HashMap, HashSet};

use crate::com::{
    ast, ir::{self, TypeProvenance}, loc::Span, reporting::{Header, Label, Report}, sem::provenance::Provenance, Checker
};

impl<'src, 'e> Checker<'src, 'e> {
    pub fn create_type(&mut self, ty: ir::Type, span: Option<Span>) -> ir::TypeID {
        let id = ir::TypeID(self.types.len());
        self.types.push(ir::TypeNode {
            parent: id,
            ty,
            loc: span.map(|s| s.wrap(self.file)),
            provenances: Vec::new(),
            depth: self.scope.depth(),
        });
        id
    }

    pub fn create_fresh_type(&mut self, span: Option<Span>) -> ir::TypeID {
        self.create_type(ir::Type::Var, span)
    }

    pub fn create_native_type(&mut self, name: &'src str, ty: ir::Type) {
        let id = self.create_type(ty, None);
        self.create_user_type(name, ir::TypeInfo::Type(id));
    }

    pub fn create_user_type(&mut self, name: &'src str, info: ir::TypeInfo) -> ir::EntityID {
        let id = self.create_entity(ir::Entity::Type(info));
        self.scope.insert(name, id);
        id
    }

    fn get_type_info(&self, id: ir::EntityID) -> &ir::TypeInfo {
        match &self.entities[id.0] {
            ir::Entity::Type(i) => i,
            _ => panic!("id '{}' is not that of a type", id.0),
        }
    }

    pub fn add_type_provenance(&mut self, id: ir::TypeID, prov: TypeProvenance) {
        self.types[id.0].provenances.push(prov)
    }

    pub fn set_type_span(&mut self, id: ir::TypeID, span: Span) {
        self.types[id.0].loc = Some(span.wrap(self.file))
    }

    pub fn clone_type_repr(&mut self, ty: ir::TypeID) -> ir::TypeID {
        let ty = self.get_type_repr(ty);

        let loc = self.types[ty.0].loc;
        let provenances = self.types[ty.0].provenances.clone();

        let new_ty = self.create_type(self.types[ty.0].ty.clone(), None);
        let new_node = &mut self.types[new_ty.0];

        new_node.loc = loc;
        new_node.provenances = provenances;
        self.unify(ty, new_ty, &[]); // should never fail

        new_ty
    }

    fn get_type_repr(&mut self, id: ir::TypeID) -> ir::TypeID {
        if self.types[id.0].parent == id {
            return id;
        }

        let r = self.get_type_repr(self.types[id.0].parent);
        self.types[id.0].parent = r;
        r
    }

    fn join_type_repr(&mut self, left: ir::TypeID, right: ir::TypeID) {
        self.types[left.0].parent = right;
    }

    fn occurs_in_type(&mut self, left: ir::TypeID, right: ir::TypeID) -> bool {
        let left = self.get_type_repr(left);
        let right = self.get_type_repr(right);

        use ir::Type as T;
        match self.types[right.0].ty.clone() {
            T::Var => left == right,
            T::Int => false,
            T::Float => false,
            T::Bool => false,
            T::String => false,
            T::Tuple(items) => items.iter().any(|&item| self.occurs_in_type(left, item)),
            T::Array(item) => self.occurs_in_type(left, item),
            T::Lambda(args, ret) => {
                args.iter().any(|&arg| self.occurs_in_type(left, arg))
                    || self.occurs_in_type(left, ret)
            }
            T::Union(_, Some(items)) => items.iter().any(|&item| self.occurs_in_type(left, item)),
            T::Union(_, None) => false,
        }
    }

    pub fn unify(&mut self, left: ir::TypeID, right: ir::TypeID, provenances: &[Provenance]) {
        let repr_left = self.get_type_repr(left);
        let repr_right = self.get_type_repr(right);

        let left = &self.types[repr_left.0];
        let right = &self.types[repr_right.0];

        use ir::Type as T;
        match (left.ty.clone(), right.ty.clone()) {
            (T::Var, T::Var) => {
                self.join_type_repr(repr_left, repr_right);
                return;
            }

            (_, T::Var) => {
                if !self.occurs_in_type(repr_right, repr_left) {
                    self.join_type_repr(repr_right, repr_left);
                    return;
                }
            }
            (T::Var, _) => {
                if !self.occurs_in_type(repr_left, repr_right) {
                    self.join_type_repr(repr_left, repr_right);
                    return;
                }
            }

            (T::Int, T::Int) => return,
            (T::Float, T::Float) => return,
            (T::String, T::String) => return,
            (T::Bool, T::Bool) => return,

            (T::Tuple(left_items), T::Tuple(right_items)) => {
                if left_items.len() == right_items.len() {
                    for (&left_item, &right_item) in left_items.iter().zip(right_items.iter()) {
                        self.unify(left_item, right_item, provenances);
                    }
                    return;
                }
            }

            (T::Array(left_item), T::Array(right_item)) => {
                self.unify(left_item, right_item, provenances);
                return;
            }

            (T::Lambda(left_args, left_ret), T::Lambda(right_args, right_ret)) => {
                if left_args.len() == right_args.len() {
                    for (&left_arg, &right_arg) in left_args.iter().zip(right_args.iter()) {
                        self.unify(right_arg, left_arg, provenances);
                    }
                    self.unify(left_ret, right_ret, provenances);
                    return;
                }
            }

            (T::Union(left_union, Some(left_items)), T::Union(right_union, Some(right_items)))
                if left_union.0 == right_union.0 =>
            {
                for (left_item, right_item) in left_items.iter().zip(right_items.iter()) {
                    self.unify(*left_item, *right_item, provenances);
                }
                return;
            }

            (T::Union(left_union, None), T::Union(right_union, None))
                if left_union.0 == right_union.0 =>
            {
                return;
            }

            _ => {}
        }

        let left_string = self.get_type_string(repr_left);
        let right_string = self.get_type_string(repr_right);
        let left = &self.types[repr_left.0];
        let right = &self.types[repr_right.0];
        let left_loc = left.loc;
        let right_loc = right.loc;

        let report = Report::error(Header::TypeMismatch(
            left_string.clone(),
            right_string.clone(),
        ));

        let report = match left_loc {
            Some(loc) => report.with_primary_label(Label::Type(left_string.clone()), loc),
            None => report,
        };

        let report = match right_loc {
            Some(loc) => report.with_primary_label(Label::Type(right_string.clone()), loc),
            None => report,
        };

        let mut report = report;
        for prov in provenances {
            report = prov.apply(report)
        }
        for prov in &left.provenances {
            report = prov.apply(report)
        }
        for prov in &right.provenances {
            report = prov.apply(report)
        }

        self.reports.push(report);
    }

    fn collect_type_variables(&mut self, ty: ir::TypeID, ids: &mut HashSet<ir::TypeID>) {
        let id = self.get_type_repr(ty);
        let node = &self.types[id.0];

        use ir::Type as T;
        match node.ty.clone() {
            T::Var => {
                if node.depth >= self.scope.depth() {
                    ids.insert(id);
                }
            }
            T::Int => {}
            T::Float => {}
            T::Bool => {}
            T::String => {}
            T::Tuple(items) => {
                for item in items {
                    self.collect_type_variables(item, ids);
                }
            }
            T::Array(item) => self.collect_type_variables(item, ids),
            T::Lambda(args, ret) => {
                for arg in args {
                    self.collect_type_variables(arg, ids);
                }
                self.collect_type_variables(ret, ids);
            }
            T::Union(_, Some(items)) => {
                for item in items {
                    self.collect_type_variables(item, ids);
                }
            }
            T::Union(_, None) => {}
        }
    }

    pub fn generalize_type(&mut self, ty: ir::TypeID) -> ir::Scheme {
        let mut scheme = ir::Scheme::mono(ty);
        self.collect_type_variables(ty, &mut scheme.forall);
        scheme
    }

    pub fn instantiate_scheme(&mut self, scheme: ir::Scheme) -> ir::TypeID {
        let sub = scheme
            .forall
            .into_iter()
            .map(|id| (id, self.create_fresh_type(None)))
            .collect();
        self.apply_type_substitution(scheme.uninstantiated, &sub)
    }

    fn apply_type_substitution(
        &mut self,
        ty: ir::TypeID,
        sub: &HashMap<ir::TypeID, ir::TypeID>,
    ) -> ir::TypeID {
        let ty = self.get_type_repr(ty);

        use ir::Type as T;
        match self.types[ty.0].ty.clone() {
            T::Var => match sub.get(&ty) {
                Some(new_ty) => *new_ty,
                None => ty,
            },
            T::Int => ty,
            T::Float => ty,
            T::Bool => ty,
            T::String => ty,
            T::Tuple(items) => {
                let new_items = items
                    .iter()
                    .map(|item| self.apply_type_substitution(*item, sub))
                    .collect();
                self.create_type(T::Tuple(new_items), None)
            }
            T::Array(item) => {
                let new_item = self.apply_type_substitution(item, sub);
                self.create_type(T::Array(new_item), None)
            }
            T::Lambda(args, ret) => {
                let new_args = args
                    .iter()
                    .map(|arg| self.apply_type_substitution(*arg, sub))
                    .collect();
                let new_ret = self.apply_type_substitution(ret, sub);
                self.create_type(T::Lambda(new_args, new_ret), None)
            }
            T::Union(eid, Some(items)) => {
                let new_items = items
                    .iter()
                    .map(|item| self.apply_type_substitution(*item, sub))
                    .collect();
                self.create_type(T::Union(eid, Some(new_items)), None)
            }
            T::Union(_, None) => ty,
        }
    }

    fn get_type_string_map(
        &mut self,
        id: ir::TypeID,
        name_map: &HashMap<ir::TypeID, String>,
    ) -> ir::TypeString {
        use ir::Type as T;
        use ir::TypeString as S;
        let repr = self.get_type_repr(id);
        match self.types[repr.0].ty.clone() {
            T::Var => match name_map.get(&repr) {
                Some(name) => S::Name(name.clone()),
                None => S::Name(format!("X{}", repr.0)),
            },
            T::Int => S::Int,
            T::Float => S::Float,
            T::Bool => S::Bool,
            T::String => S::String,
            T::Tuple(items) => S::Tuple(
                items
                    .iter()
                    .map(|item| self.get_type_string_map(*item, name_map))
                    .collect(),
            ),
            T::Array(item) => S::Array(Box::new(self.get_type_string_map(item, name_map))),
            T::Lambda(args, ret) => S::Lambda(
                args.iter()
                    .map(|arg| self.get_type_string_map(*arg, name_map))
                    .collect(),
                Box::new(self.get_type_string_map(ret, name_map)),
            ),
            T::Union(eid, Some(items)) => {
                let ir::TypeInfo::Union(info) = self.get_type_info(eid) else {
                    unreachable!()
                };
                let name = info.name.clone();
                S::Constructor(
                    name,
                    items
                        .iter()
                        .map(|item| self.get_type_string_map(*item, name_map))
                        .collect(),
                )
            }
            T::Union(eid, None) => {
                let ir::TypeInfo::Union(info) = self.get_type_info(eid) else {
                    unreachable!()
                };
                let name = info.name.clone();
                S::Name(name)
            }
        }
    }

    pub fn get_type_string(&mut self, id: ir::TypeID) -> ir::TypeString {
        self.get_type_string_map(id, &HashMap::new())
    }

    #[allow(dead_code)]
    pub fn get_scheme_string(&mut self, scheme: &ir::Scheme) -> ir::SchemeString {
        const NAMES: &str = "abcdefghijklmnopqrstuvwxyz";
        let name_map = scheme
            .forall
            .iter()
            .enumerate()
            .map(|(n, id)| {
                let (r, d) = (n % NAMES.len(), n / NAMES.len());
                let c = NAMES.as_bytes()[r] as char;
                (
                    *id,
                    match d {
                        0 => c.to_string(),
                        _ => format!("{c}{d}"),
                    },
                )
            })
            .collect();

        let uninstantiated = self.get_type_string_map(scheme.uninstantiated, &name_map);
        let forall = scheme
            .forall
            .iter()
            .map(|id| name_map.get(id).unwrap().clone())
            .collect();

        ir::SchemeString {
            uninstantiated,
            forall,
        }
    }

    pub fn check_type(&mut self, t: &ast::Expr) -> ir::TypeID {
        use ast::Expr as E;
        match t {
            E::Missing(t) => self.create_fresh_type(Some(t.span)),
            E::Var(t) => self.check_var_type(t),
            E::Tuple(t) => self.check_tuple_type(t),
            E::Call(t) => self.check_call_type(t),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidType())
                        .with_primary_label(Label::Empty, t.span().wrap(self.file)),
                );
                self.create_fresh_type(Some(t.span()))
            }
        }
    }
}
