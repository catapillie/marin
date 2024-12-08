use super::provenance::Provenance;
use crate::com::{
    ast,
    ir::{self, TypeID},
    loc::Span,
    reporting::{Header, Label, Report},
    scope::Scope,
};

pub struct Checker<'src, 'e> {
    source: &'src str,
    file: usize,
    reports: &'e mut Vec<Report>,

    scope: Scope<'src, ir::EntityID>,
    entities: Vec<ir::Entity>,
    types: Vec<ir::TypeNode>,
}

impl<'src, 'e> Checker<'src, 'e> {
    pub fn new(source: &'src str, file: usize, reports: &'e mut Vec<Report>) -> Self {
        Self {
            source,
            file,
            reports,

            scope: Scope::root(),
            entities: Vec::new(),
            types: Vec::new(),
        }
    }

    fn open_scope(&mut self) {
        self.scope.open(false);
    }

    fn close_scope(&mut self) {
        self.scope.close();
    }

    fn create_type(&mut self, ty: ir::Type, span: Option<Span>) -> ir::TypeID {
        let id = ir::TypeID(self.types.len());
        self.types.push(ir::TypeNode {
            parent: id,
            ty,
            loc: span.map(|s| s.wrap(self.file)),
        });
        id
    }

    fn create_fresh_type(&mut self, span: Option<Span>) -> ir::TypeID {
        self.create_type(ir::Type::Var, span)
    }

    fn get_type_repr(&mut self, id: ir::TypeID) -> TypeID {
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

        let left = &self.types[repr_left.0];
        let right = &self.types[repr_right.0];

        let left_loc = left.loc;
        let right_loc = right.loc;

        let left_string = self.get_type_string(repr_left);
        let right_string = self.get_type_string(repr_right);

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

    // fn add_entity(&mut self, entity: ir::Entity) -> ir::EntityID {
    //     let id = self.entities.len();
    //     self.entities.push(entity);
    //     ir::EntityID(id)
    // }

    fn get_entity(&self, id: ir::EntityID) -> &ir::Entity {
        &self.entities[id.0]
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
            E::Loop(..) => todo!(),
            E::Conditional(..) => todo!(),
            E::Break(..) => todo!(),
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

    fn check_block(&mut self, e: &ast::Block) -> ir::CheckedExpr {
        let mut iter = e.items.iter().peekable();
        let mut stmts = Vec::with_capacity(e.items.len());
        let mut last = None;

        self.open_scope();

        while let Some(item) = iter.next() {
            let s = self.check_statement(item);
            if iter.peek().is_none() {
                if let ir::Stmt::Expr(e, ty) = s {
                    last = Some((e, ty))
                }
                continue;
            }
            stmts.push(s);
        }

        self.close_scope();

        let (last, last_type) = last.unwrap_or((
            ir::Expr::unit(),
            self.create_type(ir::Type::unit(), Some(e.span())),
        ));
        (ir::Expr::Block(stmts.into(), Box::new(last)), last_type)
    }
}
