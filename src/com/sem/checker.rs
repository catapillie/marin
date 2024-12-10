use super::provenance::Provenance;
use crate::com::{
    ast,
    ir::{self, TypeProvenance},
    loc::Span,
    reporting::{Header, Label, Report},
    scope::Scope,
};

pub struct Checker<'src, 'e> {
    source: &'src str,
    file: usize,
    reports: &'e mut Vec<Report>,

    scope: Scope<'src, ir::EntityID>,
    label_scope: Scope<'src, ir::LabelID>,

    entities: Vec<ir::Entity>,
    labels: Vec<ir::Label>,
    types: Vec<ir::TypeNode>,
}

impl<'src, 'e> Checker<'src, 'e> {
    pub fn new(source: &'src str, file: usize, reports: &'e mut Vec<Report>) -> Self {
        Self {
            source,
            file,
            reports,

            scope: Scope::root(),
            label_scope: Scope::root(),

            entities: Vec::new(),
            labels: Vec::new(),
            types: Vec::new(),
        }
    }

    fn open_scope(&mut self, blocking: bool) {
        self.scope.open(false);
        self.label_scope.open(blocking);
    }

    fn close_scope(&mut self) {
        self.scope.close();
        self.label_scope.close();
    }

    fn create_type(&mut self, ty: ir::Type, span: Option<Span>) -> ir::TypeID {
        let id = ir::TypeID(self.types.len());
        self.types.push(ir::TypeNode {
            parent: id,
            ty,
            loc: span.map(|s| s.wrap(self.file)),
            provenances: Vec::new(),
        });
        id
    }

    fn create_fresh_type(&mut self, span: Option<Span>) -> ir::TypeID {
        self.create_type(ir::Type::Var, span)
    }

    fn add_type_provenance(&mut self, id: ir::TypeID, prov: TypeProvenance) {
        self.types[id.0].provenances.push(prov)
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
        }
    }

    fn unify(&mut self, left: ir::TypeID, right: ir::TypeID, provenances: &[Provenance]) {
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
                if !self.occurs_in_type(repr_left, repr_right) {
                    self.join_type_repr(repr_right, repr_left);
                    return;
                }
            }
            (T::Var, _) => {
                if !self.occurs_in_type(repr_right, repr_left) {
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

    fn get_type_string(&mut self, id: ir::TypeID) -> ir::TypeString {
        use ir::Type as T;
        use ir::TypeString as S;
        let repr = self.get_type_repr(id).0;
        match self.types[repr].ty.clone() {
            T::Var => S::Name(format!("X{repr}")),
            T::Int => S::Int,
            T::Float => S::Float,
            T::Bool => S::Bool,
            T::String => S::String,
            T::Tuple(items) => S::Tuple(
                items
                    .iter()
                    .map(|item| self.get_type_string(*item))
                    .collect(),
            ),
            T::Array(item) => S::Array(Box::new(self.get_type_string(item))),
            T::Lambda(args, ret) => S::Lambda(
                args.iter().map(|arg| self.get_type_string(*arg)).collect(),
                Box::new(self.get_type_string(ret)),
            ),
        }
    }

    fn add_entity(&mut self, entity: ir::Entity) -> ir::EntityID {
        let id = self.entities.len();
        self.entities.push(entity);
        ir::EntityID(id)
    }

    fn get_entity(&self, id: ir::EntityID) -> &ir::Entity {
        &self.entities[id.0]
    }

    fn add_label(&mut self, label: ir::Label) -> ir::LabelID {
        let id = self.labels.len();
        self.labels.push(label);
        ir::LabelID(id)
    }

    fn get_label(&self, id: ir::LabelID) -> &ir::Label {
        &self.labels[id.0]
    }

    pub fn check_file(&mut self, ast: &ast::File) -> ir::File {
        let stmts = ast.0.iter().map(|e| self.check_statement(e)).collect();
        ir::File { stmts }
    }

    fn check_statement(&mut self, e: &ast::Expr) -> ir::Stmt {
        use ast::Expr as E;
        match e {
            E::Let(..) => todo!(),
            E::Import(..) => todo!(),
            _ => {
                let (expr, ty) = self.check_expression(e);
                ir::Stmt::Expr(expr, ty)
            }
        }
    }

    fn check_expression_list<'a>(
        &mut self,
        iter: impl IntoIterator<Item = &'a ast::Expr>,
    ) -> (Vec<ir::Expr>, Vec<ir::TypeID>) {
        iter.into_iter().map(|e| self.check_expression(e)).unzip()
    }

    fn check_expression(&mut self, e: &ast::Expr) -> ir::CheckedExpr {
        use ast::Expr as E;
        match e {
            E::Missing(_) => self.check_missing(),
            E::Int(e) => self.check_int(e),
            E::Float(e) => self.check_float(e),
            E::String(e) => self.check_string(e),
            E::True(e) => self.check_bool(e, true),
            E::False(e) => self.check_bool(e, false),
            E::Var(e) => self.check_var(e),
            E::Tuple(e) => self.check_tuple(e),
            E::Array(e) => self.check_array(e),
            E::Block(e) => self.check_block(e),
            E::Loop(e) => self.check_loop(e),
            E::Conditional(e) => self.check_conditional(e),
            E::Break(e) => self.check_break(e),
            E::Skip(..) => todo!(),
            E::Call(..) => todo!(),
            E::Access(..) => todo!(),
            E::Fun(..) => todo!(),
            _ => {
                self.reports.push(
                    Report::error(Header::InvalidExpression())
                        .with_primary_label(Label::Empty, e.span().wrap(self.file)),
                );
                self.check_missing()
            }
        }
    }

    fn check_missing(&mut self) -> ir::CheckedExpr {
        (ir::Expr::Missing, self.create_fresh_type(None))
    }

    fn check_int(&mut self, e: &ast::Lexeme) -> ir::CheckedExpr {
        let lexeme = e.span.lexeme(self.source);
        match lexeme.parse::<i64>() {
            Ok(n) => (
                ir::Expr::Int(n),
                self.create_type(ir::Type::Int, Some(e.span)),
            ),
            Err(_) => {
                self.reports.push(
                    Report::error(Header::InvalidInteger())
                        .with_primary_label(Label::Empty, e.span.wrap(self.file)),
                );
                (
                    ir::Expr::Missing,
                    self.create_type(ir::Type::Int, Some(e.span)),
                )
            }
        }
    }

    fn check_float(&mut self, e: &ast::Lexeme) -> ir::CheckedExpr {
        let lexeme = e.span.lexeme(self.source);
        match lexeme.parse::<f64>() {
            Ok(f) => (
                ir::Expr::Float(f),
                self.create_type(ir::Type::Float, Some(e.span)),
            ),
            Err(_) => {
                self.reports.push(
                    Report::error(Header::InvalidFloat())
                        .with_primary_label(Label::Empty, e.span.wrap(self.file)),
                );
                (
                    ir::Expr::Missing,
                    self.create_type(ir::Type::Float, Some(e.span)),
                )
            }
        }
    }

    fn check_string(&mut self, e: &ast::Lexeme) -> ir::CheckedExpr {
        let lit = &self.source[(e.span.start + 1)..(e.span.end - 1)];
        (
            ir::Expr::String(lit.to_string()),
            self.create_type(ir::Type::String, Some(e.span)),
        )
    }

    fn check_bool(&mut self, e: &ast::Lexeme, b: bool) -> ir::CheckedExpr {
        (
            ir::Expr::Bool(b),
            self.create_type(ir::Type::Bool, Some(e.span)),
        )
    }

    fn check_var(&mut self, e: &ast::Lexeme) -> ir::CheckedExpr {
        let name = e.span.lexeme(self.source);
        let Some(&id) = self.scope.search(name) else {
            self.reports.push(
                Report::error(Header::UnknownVariable(name.to_string()))
                    .with_primary_label(Label::Empty, e.span.wrap(self.file)),
            );
            return self.check_missing();
        };

        let ir::Entity::Variable(var) = self.get_entity(id) else {
            self.reports.push(
                Report::error(Header::NotVariable(name.to_string()))
                    .with_primary_label(Label::Empty, e.span.wrap(self.file)),
            );
            return self.check_missing();
        };

        (ir::Expr::Var(id), var.ty)
    }

    fn check_tuple(&mut self, e: &ast::Tuple) -> ir::CheckedExpr {
        if e.items.len() == 1 {
            return self.check_expression(&e.items[0]);
        }

        let (items, item_types) = self.check_expression_list(&e.items);
        (
            ir::Expr::Tuple(items.into()),
            self.create_type(ir::Type::Tuple(item_types.into()), Some(e.span())),
        )
    }

    fn check_array(&mut self, e: &ast::Array) -> ir::CheckedExpr {
        let array_item_type = self.create_fresh_type(None);
        let (items, item_types) = self.check_expression_list(&e.items);

        let provenances = &[Provenance::ArrayItems(e.span().wrap(self.file))];
        for item_type in item_types {
            self.unify(item_type, array_item_type, provenances);
        }

        (
            ir::Expr::Array(items.into()),
            self.create_type(ir::Type::Array(array_item_type), Some(e.span())),
        )
    }

    fn check_expression_block(
        &mut self,
        label: &ast::Label,
        items: &[ast::Expr],
        span: Span,
    ) -> (Box<[ir::Stmt]>, ir::LabelID, ir::TypeID) {
        let mut iter = items.iter().peekable();
        let mut stmts = Vec::with_capacity(items.len());
        let mut last_type = self.create_type(ir::Type::unit(), Some(span));

        self.open_scope(false);
        let label_id = self.check_label_definition(label, false);

        while let Some(item) = iter.next() {
            let s = self.check_statement(item);
            if iter.peek().is_none() {
                if let ir::Stmt::Expr(_, ty) = &s {
                    last_type = *ty;
                }
            }
            stmts.push(s);
        }

        self.close_scope();

        let label_info = self.get_label(label_id);
        let provenances = &[Provenance::LabelValues(
            label_info.loc,
            label_info.name.clone(),
        )];
        self.unify(last_type, label_info.ty, provenances);

        (stmts.into(), label_id, last_type)
    }

    fn check_statement_block(
        &mut self,
        label: &ast::Label,
        items: &[ast::Expr],
    ) -> (Box<[ir::Stmt]>, ir::LabelID, ir::TypeID) {
        self.open_scope(false);
        let label_id = self.check_label_definition(label, false);

        let mut stmts = Vec::with_capacity(items.len());
        for item in items {
            let s = self.check_statement(item);
            stmts.push(s);
        }

        self.close_scope();

        (stmts.into(), label_id, self.get_label(label_id).ty)
    }

    fn check_block(&mut self, e: &ast::Block) -> ir::CheckedExpr {
        let (stmts, label_id, last_type) =
            self.check_expression_block(&e.label, &e.items, e.span());
        (ir::Expr::Block(stmts, label_id), last_type)
    }

    fn check_loop(&mut self, e: &ast::Loop) -> ir::CheckedExpr {
        let (stmts, label_id, loop_type) = self.check_statement_block(&e.label, &e.items);
        (ir::Expr::Loop(stmts, label_id), loop_type)
    }

    fn check_conditional(&mut self, e: &ast::Conditional) -> ir::CheckedExpr {
        let mut branches = Vec::with_capacity(e.else_branches.len() + 1);
        let mut branch_types = Vec::with_capacity(e.else_branches.len() + 1);
        let (first_branch, first_branch_type, mut is_exhaustive) =
            self.check_branch(&e.first_branch);
        branches.push(first_branch);
        branch_types.push(first_branch_type);

        let mut exhaustive_branches_span = e.first_branch.span();
        let mut exhaustive_branch_count = 0;
        let mut unreachable_branches_span = None;
        let mut unreachable_branch_count = 0;
        for (else_tok, else_branch) in &e.else_branches {
            let branch_span = Span::combine(*else_tok, else_branch.span());
            let (branch, else_branch_type, is_else_exhaustive) = self.check_branch(else_branch);
            branch_types.push(else_branch_type);

            if !is_exhaustive {
                branches.push(branch);
                exhaustive_branches_span = Span::combine(exhaustive_branches_span, branch_span);
                exhaustive_branch_count += 1;
                is_exhaustive |= is_else_exhaustive;
            } else {
                unreachable_branch_count += 1;
                unreachable_branches_span = match unreachable_branches_span {
                    Some(sp) => Some(Span::combine(sp, branch_span)),
                    None => Some(branch_span),
                }
            }
        }

        if let Some(branches_span) = unreachable_branches_span {
            self.reports.push(
                Report::warning(Header::UnreachableConditionalBranches(
                    unreachable_branch_count,
                ))
                .with_primary_label(
                    Label::UnreachableConditionalBranches(unreachable_branch_count),
                    branches_span.wrap(self.file),
                )
                .with_secondary_label(
                    Label::ExhaustiveConditionalBranches(exhaustive_branch_count),
                    exhaustive_branches_span.wrap(self.file),
                ),
            );
        }

        let result_type = if is_exhaustive {
            let result_type = self.create_fresh_type(Some(e.span()));
            let provenances = &[Provenance::ConditionalReturnValues(
                e.span().wrap(self.file),
            )];
            for branch_type in branch_types {
                self.unify(branch_type, result_type, provenances);
            }
            result_type
        } else {
            let unit_type = self.create_type(ir::Type::unit(), Some(e.span()));
            self.add_type_provenance(
                unit_type,
                TypeProvenance::NonExhaustiveConditional(e.span().wrap(self.file)),
            );
            unit_type
        };

        (
            ir::Expr::Conditional(branches.into(), is_exhaustive),
            result_type,
        )
    }

    // returns (branch_type, is_exhaustive)
    fn check_branch(&mut self, b: &ast::Branch) -> (ir::Branch, ir::TypeID, bool) {
        use ast::Branch as B;
        let span = b.span();
        match b {
            B::If(b) => self.check_if(b, span),
            B::While(b) => self.check_while(b, span),
            B::Match(_) => todo!(),
            B::Else(b) => self.check_else(b, span),
        }
    }

    fn check_if(&mut self, b: &ast::IfBranch, span: Span) -> (ir::Branch, ir::TypeID, bool) {
        let (condition, condition_type) = self.check_expression(&b.condition);
        let bool_type = self.create_type(ir::Type::Bool, None);
        let provenances = &[Provenance::ConditionalBoolType(
            b.condition.span().wrap(self.file),
        )];
        self.unify(condition_type, bool_type, provenances);

        let (stmts, label_id, branch_type) = self.check_expression_block(&b.label, &b.body, span);
        let branch = ir::Branch::If(Box::new(condition), stmts, label_id);
        (branch, branch_type, false)
    }

    fn check_while(&mut self, b: &ast::WhileBranch, _: Span) -> (ir::Branch, ir::TypeID, bool) {
        let (condition, condition_type) = self.check_expression(&b.condition);
        let bool_type = self.create_type(ir::Type::Bool, None);
        let provenances = &[Provenance::ConditionalBoolType(
            b.condition.span().wrap(self.file),
        )];
        self.unify(condition_type, bool_type, provenances);

        let (stmts, label_id, branch_type) = self.check_statement_block(&b.label, &b.body);
        let branch = ir::Branch::While(Box::new(condition), stmts, label_id);
        (branch, branch_type, false)
    }

    fn check_else(&mut self, b: &ast::ElseBranch, span: Span) -> (ir::Branch, ir::TypeID, bool) {
        let (stmts, label_id, branch_type) = self.check_expression_block(&b.label, &b.body, span);
        let branch = ir::Branch::Else(stmts, label_id);
        (branch, branch_type, true)
    }

    fn check_break(&mut self, e: &ast::Break) -> ir::CheckedExpr {
        let (value, ty) = e
            .expr
            .as_ref()
            .map(|val| {
                let (e, t) = self.check_expression(val);
                (Some(e), t)
            })
            .unwrap_or((None, self.create_type(ir::Type::unit(), Some(e.span()))));

        let label_name = self.check_label_name(&e.label);
        let Some(label_id) = self.find_label_by_name(label_name) else {
            let name = label_name.map(str::to_string);
            self.reports.push(
                Report::error(Header::InvalidBreak(name.clone()))
                    .with_primary_label(Label::NoBreakpointFound(name), e.span().wrap(self.file)),
            );
            return self.check_missing();
        };

        self.add_type_provenance(
            ty,
            TypeProvenance::ReturnedFromBreak(
                e.span().wrap(self.file),
                self.get_label(label_id).name.clone(),
            ),
        );

        let label_info = self.get_label(label_id);
        let provenances = &[Provenance::LabelValues(
            label_info.loc,
            label_info.name.clone(),
        )];
        self.unify(ty, label_info.ty, provenances);

        (
            ir::Expr::Break(value.map(Box::new), label_id),
            self.create_fresh_type(Some(e.span())),
        )
    }

    fn check_label_definition(&mut self, l: &ast::Label, skippable: bool) -> ir::LabelID {
        let name = self.check_label_name(l);
        let ty = self.create_fresh_type(None);
        let id = self.add_label(ir::Label {
            name: name.map(str::to_string),
            ty,
            skippable,
            loc: l.span().wrap(self.file),
        });
        self.label_scope.insert(name.unwrap_or(""), id);
        id
    }

    fn check_label_name(&mut self, l: &ast::Label) -> Option<&'src str> {
        use ast::Expr as E;
        use ast::Label as L;
        match l {
            L::Empty(_) => None,
            L::Named(label) => match &*label.name_expr {
                E::Int(e) | E::Var(e) => Some(e.span.lexeme(self.source)),
                E::String(e) => Some(&self.source[(e.span.start + 1)..(e.span.end - 1)]),
                _ => {
                    self.reports.push(
                        Report::error(Header::InvalidLabel())
                            .with_primary_label(Label::Empty, l.span().wrap(self.file)),
                    );
                    None
                }
            },
        }
    }

    fn find_label_by_name(&mut self, label_name: Option<&str>) -> Option<ir::LabelID> {
        self.label_scope
            .find(|&id| {
                let name = &self.get_label(id).name;
                match (&label_name, name) {
                    (Some(query), Some(found)) => query == found,
                    (None, _) => true,
                    _ => false,
                }
            })
            .copied()
    }
}
