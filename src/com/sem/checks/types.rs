use crate::com::{
    Checker, ast,
    ir::{self, TypeProvenance},
    loc::{Loc, Span},
    reporting::{Header, Label, Report},
    sem::provenance::Provenance,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};

impl<'src> Checker<'src, '_> {
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
        self.create_user_type(name, id);
    }

    pub fn create_user_type(&mut self, name: &'src str, id: ir::TypeID) -> ir::UserTypeID {
        let id = self.entities.create_user_type(ir::UserTypeInfo { id });
        self.scope.insert(name, id.wrap());
        id
    }

    pub fn declare_type_argument(&mut self, e: &ast::Expr) -> (ir::TypeID, Option<String>) {
        use ast::Expr as E;
        match e {
            E::Var(e) => {
                let arg_span = e.span;
                let arg_name = arg_span.lexeme(self.source);
                let arg_id = self.create_fresh_type(Some(e.span));
                self.create_user_type(arg_name, arg_id);
                (arg_id, Some(arg_name.to_string()))
            }
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidTypeArg())
                        .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                );
                (self.create_fresh_type(Some(e.span())), None)
            }
        }
    }

    pub fn add_type_provenance(&mut self, id: ir::TypeID, prov: TypeProvenance) {
        self.types[id.0].provenances.push(prov)
    }

    pub fn type_depth(&mut self, id: ir::TypeID) -> usize {
        let id = self.get_type_repr(id);
        self.types[id.0].depth
    }

    pub fn set_type_span(&mut self, id: ir::TypeID, span: Span) {
        self.types[id.0].loc = Some(span.wrap(self.file))
    }

    pub fn set_type_loc(&mut self, id: ir::TypeID, loc: Loc) {
        self.types[id.0].loc = Some(loc)
    }

    pub fn clone_type_repr(&mut self, ty: ir::TypeID) -> ir::TypeID {
        let ty = self.get_type_repr(ty);

        let loc = self.types[ty.0].loc;
        let provenances = self.types[ty.0].provenances.clone();
        let depth = self.types[ty.0].depth;

        let new_ty = self.create_type(self.types[ty.0].ty.clone(), None);
        let new_node = &mut self.types[new_ty.0];

        new_node.loc = loc;
        new_node.provenances = provenances;
        new_node.depth = depth;
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

    fn get_type_repr_immut(&self, id: ir::TypeID) -> ir::TypeID {
        match self.types[id.0].parent == id {
            true => id,
            false => self.get_type_repr_immut(self.types[id.0].parent),
        }
    }

    fn join_type_repr(&mut self, left: ir::TypeID, right: ir::TypeID) {
        self.types[left.0].parent = right;
    }

    pub fn is_concrete_type(&mut self, ty: ir::TypeID) -> bool {
        let ty = self.get_type_repr(ty);
        use ir::Type as T;
        match self.types[ty.0].ty.clone() {
            T::Var => false,
            T::Int => true,
            T::Float => true,
            T::Bool => true,
            T::String => true,
            T::Tuple(items) => items.iter().all(|item| self.is_concrete_type(*item)),
            T::Array(item) => self.is_concrete_type(item),
            T::Lambda(args, ret) => {
                args.iter().all(|arg| self.is_concrete_type(*arg)) && self.is_concrete_type(ret)
            }
            T::Record(_, None) => true,
            T::Record(_, Some(items)) => items.iter().all(|item| self.is_concrete_type(*item)),
            T::Union(_, None) => true,
            T::Union(_, Some(items)) => items.iter().all(|item| self.is_concrete_type(*item)),
        }
    }

    pub fn is_relevant_type(&mut self, ty: ir::TypeID) -> bool {
        let ty = self.get_type_repr(ty);
        let node = &self.types[ty.0];

        use ir::Type as T;
        match node.ty.clone() {
            T::Var => node.depth >= self.scope.depth(),
            T::Int => false,
            T::Float => false,
            T::Bool => false,
            T::String => false,
            T::Tuple(items) => items.iter().any(|item| self.is_relevant_type(*item)),
            T::Array(item) => self.is_concrete_type(item),
            T::Lambda(args, ret) => {
                args.iter().any(|arg| self.is_concrete_type(*arg)) || self.is_concrete_type(ret)
            }
            T::Record(_, None) => true,
            T::Record(_, Some(items)) => items.iter().any(|item| self.is_concrete_type(*item)),
            T::Union(_, None) => true,
            T::Union(_, Some(items)) => items.iter().any(|item| self.is_concrete_type(*item)),
        }
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
            T::Record(_, Some(items)) => items.iter().any(|&item| self.occurs_in_type(left, item)),
            T::Record(_, None) => false,
            T::Union(_, Some(items)) => items.iter().any(|&item| self.occurs_in_type(left, item)),
            T::Union(_, None) => false,
        }
    }

    fn propagate_lower_depth(&mut self, id: ir::TypeID, new_depth: usize) {
        let ty = self.get_type_repr(id);
        let node = &mut self.types[ty.0];

        use ir::Type as T;
        match node.ty.clone() {
            T::Var => node.depth = node.depth.min(new_depth),
            T::Int => {}
            T::Float => {}
            T::Bool => {}
            T::String => {}
            T::Tuple(items) => {
                for item in items {
                    self.propagate_lower_depth(item, new_depth);
                }
            }
            T::Array(item) => self.propagate_lower_depth(item, new_depth),
            T::Lambda(args, ret) => {
                for arg in args {
                    self.propagate_lower_depth(arg, new_depth);
                }
                self.propagate_lower_depth(ret, new_depth)
            }
            T::Record(_, None) => {}
            T::Record(_, Some(items)) => {
                for item in items {
                    self.propagate_lower_depth(item, new_depth);
                }
            }
            T::Union(_, None) => {}
            T::Union(_, Some(items)) => {
                for item in items {
                    self.propagate_lower_depth(item, new_depth);
                }
            }
        }
    }

    pub fn unify(&mut self, left: ir::TypeID, right: ir::TypeID, provenances: &[Provenance]) {
        self.try_unify(left, right, provenances, false);
    }

    pub fn try_unify(
        &mut self,
        left: ir::TypeID,
        right: ir::TypeID,
        provenances: &[Provenance],
        quiet: bool,
    ) -> bool {
        let repr_left = self.get_type_repr(left);
        let repr_right = self.get_type_repr(right);

        let left = &self.types[repr_left.0];
        let right = &self.types[repr_right.0];

        use ir::Type as T;
        match (left.ty.clone(), right.ty.clone()) {
            (T::Var, T::Var) => {
                // join types by descending depth
                let depth_left = self.type_depth(repr_left);
                let depth_right = self.type_depth(repr_right);
                if depth_left >= depth_right {
                    self.join_type_repr(repr_left, repr_right)
                } else {
                    self.join_type_repr(repr_right, repr_left)
                }
                return true;
            }

            // if a non-variable type is unified against a type variable x
            // then any type sitting deeper than x in the scope
            // must be refreshed to be sitting at the depth as x
            // so that they are not generalized where they shouldn't
            (_, T::Var) => {
                let var_depth = right.depth;
                if !self.occurs_in_type(repr_right, repr_left) {
                    self.join_type_repr(repr_right, repr_left);
                    self.propagate_lower_depth(repr_left, var_depth);
                    return true;
                }
            }
            (T::Var, _) => {
                let var_depth = left.depth;
                if !self.occurs_in_type(repr_left, repr_right) {
                    self.join_type_repr(repr_left, repr_right);
                    self.propagate_lower_depth(repr_right, var_depth);
                    return true;
                }
            }

            (T::Int, T::Int) => return true,
            (T::Float, T::Float) => return true,
            (T::String, T::String) => return true,
            (T::Bool, T::Bool) => return true,

            (T::Tuple(left_items), T::Tuple(right_items)) => {
                if left_items.len() == right_items.len() {
                    let mut all = true;
                    for (&left_item, &right_item) in left_items.iter().zip(right_items.iter()) {
                        all &= self.try_unify(left_item, right_item, provenances, quiet);
                    }
                    return all;
                }
            }

            (T::Array(left_item), T::Array(right_item)) => {
                return self.try_unify(left_item, right_item, provenances, quiet);
            }

            (T::Lambda(left_args, left_ret), T::Lambda(right_args, right_ret)) => {
                if left_args.len() == right_args.len() {
                    let mut all = true;
                    for (&left_arg, &right_arg) in left_args.iter().zip(right_args.iter()) {
                        all &= self.try_unify(right_arg, left_arg, provenances, quiet);
                    }
                    all &= self.try_unify(left_ret, right_ret, provenances, quiet);
                    return all;
                }
            }

            (T::Record(left_rec, Some(left_items)), T::Record(right_rec, Some(right_items)))
                if left_rec.0 == right_rec.0 =>
            {
                let mut all = true;
                for (left_item, right_item) in left_items.iter().zip(right_items.iter()) {
                    all &= self.try_unify(*left_item, *right_item, provenances, quiet);
                }
                return all;
            }

            (T::Record(left_rec, None), T::Record(right_rec, None))
                if left_rec.0 == right_rec.0 =>
            {
                return true;
            }

            (T::Union(left_union, Some(left_items)), T::Union(right_union, Some(right_items)))
                if left_union.0 == right_union.0 =>
            {
                let mut all = true;
                for (left_item, right_item) in left_items.iter().zip(right_items.iter()) {
                    all &= self.try_unify(*left_item, *right_item, provenances, quiet);
                }
                return all;
            }

            (T::Union(left_union, None), T::Union(right_union, None))
                if left_union.0 == right_union.0 =>
            {
                return true;
            }

            _ => {}
        }

        if quiet {
            return false;
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
        false
    }

    pub fn unify_constraint(&mut self, left: &ir::Constraint, right: &ir::Constraint) {
        debug_assert_eq!(
            left.id, right.id,
            "attempt to unify constraints of different classes"
        );
        debug_assert_eq!(left.class_args.len(), right.class_args.len());
        debug_assert_eq!(left.associated_args.len(), right.associated_args.len());

        for (left, right) in left.class_args.iter().zip(&right.class_args) {
            self.unify(*left, *right, &[]);
        }
        for (left, right) in left.associated_args.iter().zip(&right.associated_args) {
            self.unify(*left, *right, &[]);
        }
    }

    pub fn try_unify_constraint_args(
        &mut self,
        left: &ir::Constraint,
        right: &ir::Constraint,
    ) -> bool {
        debug_assert_eq!(
            left.id, right.id,
            "attempt to unify constraints of different classes"
        );
        debug_assert_eq!(left.class_args.len(), right.class_args.len());
        debug_assert_eq!(left.associated_args.len(), right.associated_args.len());

        let mut all = true;
        for (left, right) in left.class_args.iter().zip(&right.class_args) {
            all &= self.try_unify(*left, *right, &[], true);
        }
        all
    }

    fn collect_type_variables(&mut self, ty: ir::TypeID, ids: &mut BTreeSet<ir::TypeID>) {
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
            T::Record(_, Some(items)) => {
                for item in items {
                    self.collect_type_variables(item, ids);
                }
            }
            T::Record(_, None) => {}
            T::Union(_, Some(items)) => {
                for item in items {
                    self.collect_type_variables(item, ids);
                }
            }
            T::Union(_, None) => {}
        }
    }

    pub fn collect_constraint_variables(
        &mut self,
        constraint: &ir::Constraint,
        ids: &mut BTreeSet<ir::TypeID>,
    ) {
        for arg in &constraint.class_args {
            self.collect_type_variables(*arg, ids);
        }
        for arg in &constraint.associated_args {
            self.collect_type_variables(*arg, ids);
        }
    }

    pub fn generalize_type(&mut self, ty: ir::TypeID) -> ir::Scheme {
        let mut scheme = ir::Scheme::mono(ty);
        self.collect_type_variables(ty, &mut scheme.forall);
        scheme
    }

    pub fn add_class_constraint(&mut self, scheme: &mut ir::Scheme, constraint: ir::Constraint) {
        self.collect_constraint_variables(&constraint, &mut scheme.forall);
        scheme.constraints.push(constraint);
    }

    pub fn instantiate_scheme(
        &mut self,
        scheme: ir::Scheme,
        constraint_loc: Option<Loc>,
    ) -> ir::TypeID {
        let sub = self.build_type_substitution(scheme.forall);

        for constraint in scheme.constraints {
            let mut subbed = self.apply_constraint_substitution(constraint, &sub);
            subbed.loc = constraint_loc.unwrap_or(subbed.loc);
            self.require_class_constraint(subbed);
        }

        self.apply_type_substitution(scheme.uninstantiated, &sub)
    }

    pub fn instantiate_scheme_keep_constraints(
        &mut self,
        scheme: ir::Scheme,
    ) -> (ir::TypeID, Vec<ir::Constraint>) {
        let sub = self.build_type_substitution(scheme.forall);

        let mut subbed_constraints = Vec::new();
        for constraint in scheme.constraints {
            let subbed = self.apply_constraint_substitution(constraint, &sub);
            subbed_constraints.push(subbed);
        }

        (
            self.apply_type_substitution(scheme.uninstantiated, &sub),
            subbed_constraints,
        )
    }

    pub fn instantiate_instance_scheme(
        &mut self,
        scheme: ir::InstanceScheme,
    ) -> (ir::Constraint, Vec<ir::Constraint>) {
        let sub = self.build_type_substitution(scheme.forall);

        let mut subbed_constraints = Vec::new();
        for constraint in scheme.required_constraints {
            let subbed = self.apply_constraint_substitution(constraint, &sub);
            subbed_constraints.push(subbed);
        }

        let subbed = self.apply_constraint_substitution(scheme.constraint, &sub);
        (subbed, subbed_constraints)
    }

    pub fn build_type_substitution(
        &mut self,
        domain: BTreeSet<ir::TypeID>,
    ) -> BTreeMap<ir::TypeID, ir::TypeID> {
        domain
            .into_iter()
            .map(|id| (self.get_type_repr(id), self.create_fresh_type(None)))
            .collect()
    }

    pub fn apply_type_substitution(
        &mut self,
        ty: ir::TypeID,
        sub: &BTreeMap<ir::TypeID, ir::TypeID>,
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
            T::Record(eid, Some(items)) => {
                let new_items = items
                    .iter()
                    .map(|item| self.apply_type_substitution(*item, sub))
                    .collect();
                self.create_type(T::Record(eid, Some(new_items)), None)
            }
            T::Record(_, None) => ty,
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

    pub fn apply_constraint_substitution(
        &mut self,
        constraint: ir::Constraint,
        sub: &BTreeMap<ir::TypeID, ir::TypeID>,
    ) -> ir::Constraint {
        ir::Constraint {
            id: constraint.id,
            loc: constraint.loc,
            class_args: constraint
                .class_args
                .iter()
                .map(|arg| self.apply_type_substitution(*arg, sub))
                .collect(),
            associated_args: constraint
                .associated_args
                .iter()
                .map(|arg| self.apply_type_substitution(*arg, sub))
                .collect(),
        }
    }

    fn get_type_string_map(
        &self,
        id: ir::TypeID,
        name_map: &BTreeMap<ir::TypeID, String>,
        hide: bool,
    ) -> ir::TypeString {
        use ir::Type as T;
        use ir::TypeString as S;
        let repr = self.get_type_repr_immut(id);
        match &self.types[repr.0].ty {
            T::Var => match name_map.get(&repr) {
                Some(name) => S::Name(name.clone()),
                None if hide => S::Hidden,
                None => S::Name(format!("{{{}}}", repr.0)),
            },
            T::Int => S::Int,
            T::Float => S::Float,
            T::Bool => S::Bool,
            T::String => S::String,
            T::Tuple(items) => S::Tuple(
                items
                    .iter()
                    .map(|item| self.get_type_string_map(*item, name_map, hide))
                    .collect(),
            ),
            T::Array(item) => S::Array(Box::new(self.get_type_string_map(*item, name_map, hide))),
            T::Lambda(args, ret) => S::Lambda(
                args.iter()
                    .map(|arg| self.get_type_string_map(*arg, name_map, hide))
                    .collect(),
                Box::new(self.get_type_string_map(*ret, name_map, hide)),
            ),
            T::Record(eid, Some(items)) => {
                let info = self.entities.get_record_info(*eid);
                let name = info.name.clone();
                S::Constructor(
                    name,
                    items
                        .iter()
                        .map(|item| self.get_type_string_map(*item, name_map, hide))
                        .collect(),
                )
            }
            T::Record(eid, None) => {
                let info = self.entities.get_record_info(*eid);
                let name = info.name.clone();
                S::Name(name)
            }
            T::Union(eid, Some(items)) => {
                let info = self.entities.get_union_info(*eid);
                let name = info.name.clone();
                S::Constructor(
                    name,
                    items
                        .iter()
                        .map(|item| self.get_type_string_map(*item, name_map, hide))
                        .collect(),
                )
            }
            T::Union(eid, None) => {
                let info = self.entities.get_union_info(*eid);
                let name = info.name.clone();
                S::Name(name)
            }
        }
    }

    pub fn get_type_string(&self, id: ir::TypeID) -> ir::TypeString {
        self.get_type_string_map(id, &BTreeMap::new(), true)
    }

    #[allow(dead_code)]
    pub fn get_type_string_detailed(&self, id: ir::TypeID) -> ir::TypeString {
        self.get_type_string_map(id, &BTreeMap::new(), false)
    }

    fn create_domain_name_map(
        &self,
        domain: &BTreeSet<ir::TypeID>,
    ) -> (BTreeMap<ir::TypeID, String>, Vec<ir::TypeID>) {
        // update the domain to use representant types
        let domain = domain
            .iter()
            .map(|x| self.get_type_repr_immut(*x))
            .collect::<Vec<_>>();

        const NAMES: &str = "abcdefghijklmnopqrstuvwxyz";
        let name_map = domain
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

        (name_map, domain)
    }

    pub fn get_scheme_string(&self, scheme: &ir::Scheme) -> ir::SchemeString {
        let (name_map, domain) = self.create_domain_name_map(&scheme.forall);
        let uninstantiated = self.get_type_string_map(scheme.uninstantiated, &name_map, true);

        let forall = domain
            .iter()
            .map(|id| name_map.get(id).unwrap().clone())
            .collect();

        let mut constraints = HashSet::new();
        for constraint in &scheme.constraints {
            constraints.insert(self.get_constraint_string_map(constraint, &name_map, true));
        }
        let constraints = constraints.into_iter().collect();

        ir::SchemeString {
            uninstantiated,
            forall,
            constraints,
        }
    }

    pub fn get_instance_scheme_string(
        &self,
        scheme: &ir::InstanceScheme,
    ) -> ir::InstanceSchemeString {
        let (name_map, domain) = self.create_domain_name_map(&scheme.forall);
        let constraint = self.get_constraint_string_map(&scheme.constraint, &name_map, true);

        let forall = domain
            .iter()
            .map(|id| name_map.get(id).unwrap().clone())
            .collect();

        let mut required_constraints = HashSet::new();
        for constraint in &scheme.required_constraints {
            required_constraints
                .insert(self.get_constraint_string_map(constraint, &name_map, true));
        }
        let required_constraints = required_constraints.into_iter().collect();

        ir::InstanceSchemeString {
            forall,
            constraint,
            required_constraints,
        }
    }

    pub fn get_constraint_string_map(
        &self,
        constraint: &ir::Constraint,
        name_map: &BTreeMap<ir::TypeID, String>,
        hide: bool,
    ) -> ir::ConstraintString {
        ir::ConstraintString {
            name: self.entities.get_class_info(constraint.id).name.to_string(),
            class_args: constraint
                .class_args
                .iter()
                .map(|arg| self.get_type_string_map(*arg, name_map, hide))
                .collect(),
            associated_args: constraint
                .associated_args
                .iter()
                .map(|arg| self.get_type_string_map(*arg, name_map, hide))
                .collect(),
        }
    }

    pub fn get_constraint_string(&self, constraint: &ir::Constraint) -> ir::ConstraintString {
        let name_map = Default::default();
        self.get_constraint_string_map(constraint, &name_map, true)
    }

    #[allow(dead_code)]
    pub fn get_constraint_string_detailed(
        &self,
        constraint: &ir::Constraint,
    ) -> ir::ConstraintString {
        let name_map = Default::default();
        self.get_constraint_string_map(constraint, &name_map, false)
    }

    pub fn check_type(&mut self, t: &ast::Expr) -> ir::TypeID {
        use ast::Expr as E;
        match t {
            E::Missing(t) => self.create_fresh_type(Some(t.span)),
            E::Var(t) => self.check_var_type(t),
            E::Tuple(t) => self.check_tuple_type(t),
            E::Call(t) => self.check_call_type(t),
            E::Fun(t) => self.check_fun_type(t),
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
