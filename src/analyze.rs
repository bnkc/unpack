use rustpython_parser::ast;

trait Visitor {
    fn visit_expr(&mut self, node: &ast::Expr);
    fn visit_binop(&mut self, node: &ast::ExprBinOp);
    fn visit_fn(&mut self, node: &ast::StmtFunctionDef);
    // ... other visit methods for different node types
}

pub struct TypeChecker;

impl TypeChecker {
    pub fn visit_mod(&mut self, module: &ast::Mod) {
        if let ast::Mod::Module(stmts) = module {
            for stmt in stmts.body.iter() {
                self.visit_stmt(stmt);
            }
        } else {
            unimplemented!("Module type not implemented....YET!");
        }
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::Assign(assign) => self.visit_expr(&assign.value),
            ast::Stmt::FunctionDef(func) => self.visit_fn(&func),
            // ... handle other statement types
            _ => (),
        }
    }

    fn get_constant_type(&self, expr_constant: &ast::ExprConstant) -> Option<&str> {
        let constant = &expr_constant.value;
        match constant {
            ast::Constant::Int(_) => Some("int"),
            ast::Constant::Float(_) => Some("float"),
            ast::Constant::Str(_) => Some("str"),
            // ... handle other constant types
            _ => None,
        }
    }
}

impl Visitor for TypeChecker {
    fn visit_fn(&mut self, node: &ast::StmtFunctionDef) {
        if let Some(returns) = &node.returns {
            match &**returns {
                ast::Expr::Name(name) => {
                    println!("Return type: {:#?}", name.id);
                }
                _ => unimplemented!("Return type not implemented....YET!"),
            }
        }
    }

    fn visit_expr(&mut self, node: &ast::Expr) {
        match node {
            ast::Expr::BinOp(binop) => self.visit_binop(binop),
            _ => unimplemented!("Expression {:#?} not implemented....YET!", node),
        }
    }

    fn visit_binop(&mut self, node: &ast::ExprBinOp) {
        let left_type: Option<&str> = match &*node.left {
            ast::Expr::Constant(c) => self.get_constant_type(c),
            // ... handle other expression types
            _ => unimplemented!("Expression type not implemented....YET!"),
        };
        let right_type: Option<&str> = match &*node.right {
            ast::Expr::Constant(c) => self.get_constant_type(c),
            // ... handle other expression types
            _ => unimplemented!("Expression type not implemented....YET!"),
        };

        match (left_type, right_type) {
            (Some("int"), Some("int")) => (), // Int + Int is valid
            (Some("str"), Some("str")) => (), // Str + Str is valid
            _ => eprintln!(
                "Unsupported operand types for +: {:?} and {:?}",
                left_type, right_type
            ),
        }
    }
}
