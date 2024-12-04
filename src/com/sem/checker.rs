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

    fn unify(&mut self, left: ir::TypeID, right: ir::TypeID) {
        println!(
            "    {} = {}",
            self.get_type_string(left),
            self.get_type_string(right)
        );
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
        for item_type in item_types {
            self.unify(item_type, array_item_type);
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
