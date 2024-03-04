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
            unimplemented!("Visitng MOD broken");
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
            ast::Constant::Bool(_) => Some("bool"),
            // ... handle other constant types
            _ => None,
        }
    }
}

impl Visitor for TypeChecker {
    fn visit_fn(&mut self, node: &ast::StmtFunctionDef) {
        let mut declared_return_type: Option<&str> = None;
        if let Some(returns) = &node.returns {
            declared_return_type = Some(match &**returns {
                ast::Expr::Name(name) => &name.id,
                _ => unimplemented!("Return type not implemented....YET!"),
            });
        }
        println!("{:#?}", node)
        // let mut inferred_return_type: Option<&str> = None;
        // for stmt in node.body.iter() {
        //     match stmt {
        //         ast::Stmt::Return(ret) => {
        //             if let Some(expr) = &ret.value {
        //                 inferred_return_type = match &**expr {
        //                     ast::Expr::Constant(c) => self.get_constant_type(c),
        //                     _ => unimplemented!("Expression type not implemented....YET!"),
        //                 };
        //             }
        //         }
        //         _ => (),
        //     }
        // }
        // if let (Some(declared), Some(inferred)) = (declared_return_type, inferred_return_type) {
        //     if declared != inferred {
        //         eprintln!(
        //             "Incompatible return value type (got '{}', expected '{}') rustpy(error)
        //             ",
        //             inferred, declared
        //         );
        //     }
        // }
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
