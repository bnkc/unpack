use rustpython_parser::ast::*;
use text_size::TextRange;
#[allow(unused_variables)]
pub trait Visitor<R = TextRange> {
    fn visit_stmt(&mut self, node: Stmt<R>) {
        self.generic_visit_stmt(node)
    }
    fn generic_visit_stmt(&mut self, node: Stmt<R>) {
        match node {
            Stmt::FunctionDef(data) => self.visit_stmt_function_def(data),
            Stmt::AsyncFunctionDef(data) => self.visit_stmt_async_function_def(data),
            Stmt::ClassDef(data) => self.visit_stmt_class_def(data),
            Stmt::Return(data) => self.visit_stmt_return(data),
            Stmt::Delete(data) => self.visit_stmt_delete(data),
            Stmt::Assign(data) => self.visit_stmt_assign(data),
            Stmt::TypeAlias(data) => self.visit_stmt_type_alias(data),
            Stmt::AugAssign(data) => self.visit_stmt_aug_assign(data),
            Stmt::AnnAssign(data) => self.visit_stmt_ann_assign(data),
            Stmt::For(data) => self.visit_stmt_for(data),
            Stmt::AsyncFor(data) => self.visit_stmt_async_for(data),
            Stmt::While(data) => self.visit_stmt_while(data),
            Stmt::If(data) => self.visit_stmt_if(data),
            Stmt::With(data) => self.visit_stmt_with(data),
            Stmt::AsyncWith(data) => self.visit_stmt_async_with(data),
            Stmt::Match(data) => self.visit_stmt_match(data),
            Stmt::Raise(data) => self.visit_stmt_raise(data),
            Stmt::Try(data) => self.visit_stmt_try(data),
            Stmt::TryStar(data) => self.visit_stmt_try_star(data),
            Stmt::Assert(data) => self.visit_stmt_assert(data),
            Stmt::Import(data) => self.visit_stmt_import(data),
            Stmt::ImportFrom(data) => self.visit_stmt_import_from(data),
            Stmt::Global(data) => self.visit_stmt_global(data),
            Stmt::Nonlocal(data) => self.visit_stmt_nonlocal(data),
            Stmt::Expr(data) => self.visit_stmt_expr(data),
            Stmt::Pass(data) => self.visit_stmt_pass(data),
            Stmt::Break(data) => self.visit_stmt_break(data),
            Stmt::Continue(data) => self.visit_stmt_continue(data),
        }
    }
    fn visit_stmt_function_def(&mut self, node: StmtFunctionDef<R>) {
        self.generic_visit_stmt_function_def(node)
    }
    fn generic_visit_stmt_function_def(&mut self, node: StmtFunctionDef<R>) {
        {
            let value = node.args;
            self.visit_arguments(*value);
        }
        for value in node.body {
            self.visit_stmt(value);
        }
        for value in node.decorator_list {
            self.visit_expr(value);
        }
        if let Some(value) = node.returns {
            self.visit_expr(*value);
        }
        for value in node.type_params {
            self.visit_type_param(value);
        }
    }
    fn visit_stmt_async_function_def(&mut self, node: StmtAsyncFunctionDef<R>) {
        self.generic_visit_stmt_async_function_def(node)
    }
    fn generic_visit_stmt_async_function_def(&mut self, node: StmtAsyncFunctionDef<R>) {
        {
            let value = node.args;
            self.visit_arguments(*value);
        }
        for value in node.body {
            self.visit_stmt(value);
        }
        for value in node.decorator_list {
            self.visit_expr(value);
        }
        if let Some(value) = node.returns {
            self.visit_expr(*value);
        }
        for value in node.type_params {
            self.visit_type_param(value);
        }
    }
    fn visit_stmt_class_def(&mut self, node: StmtClassDef<R>) {
        self.generic_visit_stmt_class_def(node)
    }
    fn generic_visit_stmt_class_def(&mut self, node: StmtClassDef<R>) {
        for value in node.bases {
            self.visit_expr(value);
        }
        for value in node.keywords {
            self.visit_keyword(value);
        }
        for value in node.body {
            self.visit_stmt(value);
        }
        for value in node.decorator_list {
            self.visit_expr(value);
        }
        for value in node.type_params {
            self.visit_type_param(value);
        }
    }
    fn visit_stmt_return(&mut self, node: StmtReturn<R>) {
        self.generic_visit_stmt_return(node)
    }
    fn generic_visit_stmt_return(&mut self, node: StmtReturn<R>) {
        if let Some(value) = node.value {
            self.visit_expr(*value);
        }
    }
    fn visit_stmt_delete(&mut self, node: StmtDelete<R>) {
        self.generic_visit_stmt_delete(node)
    }
    fn generic_visit_stmt_delete(&mut self, node: StmtDelete<R>) {
        for value in node.targets {
            self.visit_expr(value);
        }
    }
    fn visit_stmt_assign(&mut self, node: StmtAssign<R>) {
        self.generic_visit_stmt_assign(node)
    }
    fn generic_visit_stmt_assign(&mut self, node: StmtAssign<R>) {
        for value in node.targets {
            self.visit_expr(value);
        }
        {
            let value = node.value;
            self.visit_expr(*value);
        }
    }
    fn visit_stmt_type_alias(&mut self, node: StmtTypeAlias<R>) {
        self.generic_visit_stmt_type_alias(node)
    }
    fn generic_visit_stmt_type_alias(&mut self, node: StmtTypeAlias<R>) {
        {
            let value = node.name;
            self.visit_expr(*value);
        }
        for value in node.type_params {
            self.visit_type_param(value);
        }
        {
            let value = node.value;
            self.visit_expr(*value);
        }
    }
    fn visit_stmt_aug_assign(&mut self, node: StmtAugAssign<R>) {
        self.generic_visit_stmt_aug_assign(node)
    }
    fn generic_visit_stmt_aug_assign(&mut self, node: StmtAugAssign<R>) {
        {
            let value = node.target;
            self.visit_expr(*value);
        }
        {
            let value = node.value;
            self.visit_expr(*value);
        }
    }
    fn visit_stmt_ann_assign(&mut self, node: StmtAnnAssign<R>) {
        self.generic_visit_stmt_ann_assign(node)
    }
    fn generic_visit_stmt_ann_assign(&mut self, node: StmtAnnAssign<R>) {
        {
            let value = node.target;
            self.visit_expr(*value);
        }
        {
            let value = node.annotation;
            self.visit_expr(*value);
        }
        if let Some(value) = node.value {
            self.visit_expr(*value);
        }
    }
    fn visit_stmt_for(&mut self, node: StmtFor<R>) {
        self.generic_visit_stmt_for(node)
    }
    fn generic_visit_stmt_for(&mut self, node: StmtFor<R>) {
        {
            let value = node.target;
            self.visit_expr(*value);
        }
        {
            let value = node.iter;
            self.visit_expr(*value);
        }
        for value in node.body {
            self.visit_stmt(value);
        }
        for value in node.orelse {
            self.visit_stmt(value);
        }
    }
    fn visit_stmt_async_for(&mut self, node: StmtAsyncFor<R>) {
        self.generic_visit_stmt_async_for(node)
    }
    fn generic_visit_stmt_async_for(&mut self, node: StmtAsyncFor<R>) {
        {
            let value = node.target;
            self.visit_expr(*value);
        }
        {
            let value = node.iter;
            self.visit_expr(*value);
        }
        for value in node.body {
            self.visit_stmt(value);
        }
        for value in node.orelse {
            self.visit_stmt(value);
        }
    }
    fn visit_stmt_while(&mut self, node: StmtWhile<R>) {
        self.generic_visit_stmt_while(node)
    }
    fn generic_visit_stmt_while(&mut self, node: StmtWhile<R>) {
        {
            let value = node.test;
            self.visit_expr(*value);
        }
        for value in node.body {
            self.visit_stmt(value);
        }
        for value in node.orelse {
            self.visit_stmt(value);
        }
    }
    fn visit_stmt_if(&mut self, node: StmtIf<R>) {
        self.generic_visit_stmt_if(node)
    }
    fn generic_visit_stmt_if(&mut self, node: StmtIf<R>) {
        {
            let value = node.test;
            self.visit_expr(*value);
        }
        for value in node.body {
            self.visit_stmt(value);
        }
        for value in node.orelse {
            self.visit_stmt(value);
        }
    }
    fn visit_stmt_with(&mut self, node: StmtWith<R>) {
        self.generic_visit_stmt_with(node)
    }
    fn generic_visit_stmt_with(&mut self, node: StmtWith<R>) {
        for value in node.items {
            self.visit_withitem(value);
        }
        for value in node.body {
            self.visit_stmt(value);
        }
    }
    fn visit_stmt_async_with(&mut self, node: StmtAsyncWith<R>) {
        self.generic_visit_stmt_async_with(node)
    }
    fn generic_visit_stmt_async_with(&mut self, node: StmtAsyncWith<R>) {
        for value in node.items {
            self.visit_withitem(value);
        }
        for value in node.body {
            self.visit_stmt(value);
        }
    }
    fn visit_stmt_match(&mut self, node: StmtMatch<R>) {
        self.generic_visit_stmt_match(node)
    }
    fn generic_visit_stmt_match(&mut self, node: StmtMatch<R>) {
        {
            let value = node.subject;
            self.visit_expr(*value);
        }
        for value in node.cases {
            self.visit_match_case(value);
        }
    }
    fn visit_stmt_raise(&mut self, node: StmtRaise<R>) {
        self.generic_visit_stmt_raise(node)
    }
    fn generic_visit_stmt_raise(&mut self, node: StmtRaise<R>) {
        if let Some(value) = node.exc {
            self.visit_expr(*value);
        }
        if let Some(value) = node.cause {
            self.visit_expr(*value);
        }
    }
    fn visit_stmt_try(&mut self, node: StmtTry<R>) {
        self.generic_visit_stmt_try(node)
    }
    fn generic_visit_stmt_try(&mut self, node: StmtTry<R>) {
        for value in node.body {
            self.visit_stmt(value);
        }
        for value in node.handlers {
            self.visit_excepthandler(value);
        }
        for value in node.orelse {
            self.visit_stmt(value);
        }
        for value in node.finalbody {
            self.visit_stmt(value);
        }
    }
    fn visit_stmt_try_star(&mut self, node: StmtTryStar<R>) {
        self.generic_visit_stmt_try_star(node)
    }
    fn generic_visit_stmt_try_star(&mut self, node: StmtTryStar<R>) {
        for value in node.body {
            self.visit_stmt(value);
        }
        for value in node.handlers {
            self.visit_excepthandler(value);
        }
        for value in node.orelse {
            self.visit_stmt(value);
        }
        for value in node.finalbody {
            self.visit_stmt(value);
        }
    }
    fn visit_stmt_assert(&mut self, node: StmtAssert<R>) {
        self.generic_visit_stmt_assert(node)
    }
    fn generic_visit_stmt_assert(&mut self, node: StmtAssert<R>) {
        {
            let value = node.test;
            self.visit_expr(*value);
        }
        if let Some(value) = node.msg {
            self.visit_expr(*value);
        }
    }
    fn visit_stmt_import(&mut self, node: StmtImport<R>) {
        self.generic_visit_stmt_import(node)
    }
    fn generic_visit_stmt_import(&mut self, node: StmtImport<R>) {
        for value in node.names {
            self.visit_alias(value);
        }
    }
    fn visit_stmt_import_from(&mut self, node: StmtImportFrom<R>) {
        self.generic_visit_stmt_import_from(node)
    }
    fn generic_visit_stmt_import_from(&mut self, node: StmtImportFrom<R>) {
        for value in node.names {
            self.visit_alias(value);
        }
    }
    fn visit_stmt_global(&mut self, node: StmtGlobal<R>) {
        self.generic_visit_stmt_global(node)
    }
    fn generic_visit_stmt_global(&mut self, node: StmtGlobal<R>) {}
    fn visit_stmt_nonlocal(&mut self, node: StmtNonlocal<R>) {
        self.generic_visit_stmt_nonlocal(node)
    }
    fn generic_visit_stmt_nonlocal(&mut self, node: StmtNonlocal<R>) {}
    fn visit_stmt_expr(&mut self, node: StmtExpr<R>) {
        self.generic_visit_stmt_expr(node)
    }
    fn generic_visit_stmt_expr(&mut self, node: StmtExpr<R>) {
        {
            let value = node.value;
            self.visit_expr(*value);
        }
    }
    fn visit_stmt_pass(&mut self, node: StmtPass<R>) {}
    fn visit_stmt_break(&mut self, node: StmtBreak<R>) {}
    fn visit_stmt_continue(&mut self, node: StmtContinue<R>) {}
    fn visit_expr(&mut self, node: Expr<R>) {
        self.generic_visit_expr(node)
    }
    fn generic_visit_expr(&mut self, node: Expr<R>) {
        match node {
            Expr::BoolOp(data) => self.visit_expr_bool_op(data),
            Expr::NamedExpr(data) => self.visit_expr_named_expr(data),
            Expr::BinOp(data) => self.visit_expr_bin_op(data),
            Expr::UnaryOp(data) => self.visit_expr_unary_op(data),
            Expr::Lambda(data) => self.visit_expr_lambda(data),
            Expr::IfExp(data) => self.visit_expr_if_exp(data),
            Expr::Dict(data) => self.visit_expr_dict(data),
            Expr::Set(data) => self.visit_expr_set(data),
            Expr::ListComp(data) => self.visit_expr_list_comp(data),
            Expr::SetComp(data) => self.visit_expr_set_comp(data),
            Expr::DictComp(data) => self.visit_expr_dict_comp(data),
            Expr::GeneratorExp(data) => self.visit_expr_generator_exp(data),
            Expr::Await(data) => self.visit_expr_await(data),
            Expr::Yield(data) => self.visit_expr_yield(data),
            Expr::YieldFrom(data) => self.visit_expr_yield_from(data),
            Expr::Compare(data) => self.visit_expr_compare(data),
            Expr::Call(data) => self.visit_expr_call(data),
            Expr::FormattedValue(data) => self.visit_expr_formatted_value(data),
            Expr::JoinedStr(data) => self.visit_expr_joined_str(data),
            Expr::Constant(data) => self.visit_expr_constant(data),
            Expr::Attribute(data) => self.visit_expr_attribute(data),
            Expr::Subscript(data) => self.visit_expr_subscript(data),
            Expr::Starred(data) => self.visit_expr_starred(data),
            Expr::Name(data) => self.visit_expr_name(data),
            Expr::List(data) => self.visit_expr_list(data),
            Expr::Tuple(data) => self.visit_expr_tuple(data),
            Expr::Slice(data) => self.visit_expr_slice(data),
        }
    }
    fn visit_expr_bool_op(&mut self, node: ExprBoolOp<R>) {
        self.generic_visit_expr_bool_op(node)
    }
    fn generic_visit_expr_bool_op(&mut self, node: ExprBoolOp<R>) {
        for value in node.values {
            self.visit_expr(value);
        }
    }
    fn visit_expr_named_expr(&mut self, node: ExprNamedExpr<R>) {
        self.generic_visit_expr_named_expr(node)
    }
    fn generic_visit_expr_named_expr(&mut self, node: ExprNamedExpr<R>) {
        {
            let value = node.target;
            self.visit_expr(*value);
        }
        {
            let value = node.value;
            self.visit_expr(*value);
        }
    }
    fn visit_expr_bin_op(&mut self, node: ExprBinOp<R>) {
        self.generic_visit_expr_bin_op(node)
    }
    fn generic_visit_expr_bin_op(&mut self, node: ExprBinOp<R>) {
        {
            let value = node.left;
            self.visit_expr(*value);
        }
        {
            let value = node.right;
            self.visit_expr(*value);
        }
    }
    fn visit_expr_unary_op(&mut self, node: ExprUnaryOp<R>) {
        self.generic_visit_expr_unary_op(node)
    }
    fn generic_visit_expr_unary_op(&mut self, node: ExprUnaryOp<R>) {
        {
            let value = node.operand;
            self.visit_expr(*value);
        }
    }
    fn visit_expr_lambda(&mut self, node: ExprLambda<R>) {
        self.generic_visit_expr_lambda(node)
    }
    fn generic_visit_expr_lambda(&mut self, node: ExprLambda<R>) {
        {
            let value = node.args;
            self.visit_arguments(*value);
        }
        {
            let value = node.body;
            self.visit_expr(*value);
        }
    }
    fn visit_expr_if_exp(&mut self, node: ExprIfExp<R>) {
        self.generic_visit_expr_if_exp(node)
    }
    fn generic_visit_expr_if_exp(&mut self, node: ExprIfExp<R>) {
        {
            let value = node.test;
            self.visit_expr(*value);
        }
        {
            let value = node.body;
            self.visit_expr(*value);
        }
        {
            let value = node.orelse;
            self.visit_expr(*value);
        }
    }
    fn visit_expr_dict(&mut self, node: ExprDict<R>) {
        self.generic_visit_expr_dict(node)
    }
    fn generic_visit_expr_dict(&mut self, node: ExprDict<R>) {
        for value in node.keys.into_iter().flatten() {
            self.visit_expr(value);
        }
        for value in node.values {
            self.visit_expr(value);
        }
    }
    fn visit_expr_set(&mut self, node: ExprSet<R>) {
        self.generic_visit_expr_set(node)
    }
    fn generic_visit_expr_set(&mut self, node: ExprSet<R>) {
        for value in node.elts {
            self.visit_expr(value);
        }
    }
    fn visit_expr_list_comp(&mut self, node: ExprListComp<R>) {
        self.generic_visit_expr_list_comp(node)
    }
    fn generic_visit_expr_list_comp(&mut self, node: ExprListComp<R>) {
        {
            let value = node.elt;
            self.visit_expr(*value);
        }
        for value in node.generators {
            self.visit_comprehension(value);
        }
    }
    fn visit_expr_set_comp(&mut self, node: ExprSetComp<R>) {
        self.generic_visit_expr_set_comp(node)
    }
    fn generic_visit_expr_set_comp(&mut self, node: ExprSetComp<R>) {
        {
            let value = node.elt;
            self.visit_expr(*value);
        }
        for value in node.generators {
            self.visit_comprehension(value);
        }
    }
    fn visit_expr_dict_comp(&mut self, node: ExprDictComp<R>) {
        self.generic_visit_expr_dict_comp(node)
    }
    fn generic_visit_expr_dict_comp(&mut self, node: ExprDictComp<R>) {
        {
            let value = node.key;
            self.visit_expr(*value);
        }
        {
            let value = node.value;
            self.visit_expr(*value);
        }
        for value in node.generators {
            self.visit_comprehension(value);
        }
    }
    fn visit_expr_generator_exp(&mut self, node: ExprGeneratorExp<R>) {
        self.generic_visit_expr_generator_exp(node)
    }
    fn generic_visit_expr_generator_exp(&mut self, node: ExprGeneratorExp<R>) {
        {
            let value = node.elt;
            self.visit_expr(*value);
        }
        for value in node.generators {
            self.visit_comprehension(value);
        }
    }
    fn visit_expr_await(&mut self, node: ExprAwait<R>) {
        self.generic_visit_expr_await(node)
    }
    fn generic_visit_expr_await(&mut self, node: ExprAwait<R>) {
        {
            let value = node.value;
            self.visit_expr(*value);
        }
    }
    fn visit_expr_yield(&mut self, node: ExprYield<R>) {
        self.generic_visit_expr_yield(node)
    }
    fn generic_visit_expr_yield(&mut self, node: ExprYield<R>) {
        if let Some(value) = node.value {
            self.visit_expr(*value);
        }
    }
    fn visit_expr_yield_from(&mut self, node: ExprYieldFrom<R>) {
        self.generic_visit_expr_yield_from(node)
    }
    fn generic_visit_expr_yield_from(&mut self, node: ExprYieldFrom<R>) {
        {
            let value = node.value;
            self.visit_expr(*value);
        }
    }
    fn visit_expr_compare(&mut self, node: ExprCompare<R>) {
        self.generic_visit_expr_compare(node)
    }
    fn generic_visit_expr_compare(&mut self, node: ExprCompare<R>) {
        {
            let value = node.left;
            self.visit_expr(*value);
        }
        for value in node.comparators {
            self.visit_expr(value);
        }
    }
    fn visit_expr_call(&mut self, node: ExprCall<R>) {
        self.generic_visit_expr_call(node)
    }
    fn generic_visit_expr_call(&mut self, node: ExprCall<R>) {
        {
            let value = node.func;
            self.visit_expr(*value);
        }
        for value in node.args {
            self.visit_expr(value);
        }
        for value in node.keywords {
            self.visit_keyword(value);
        }
    }
    fn visit_expr_formatted_value(&mut self, node: ExprFormattedValue<R>) {
        self.generic_visit_expr_formatted_value(node)
    }
    fn generic_visit_expr_formatted_value(&mut self, node: ExprFormattedValue<R>) {
        {
            let value = node.value;
            self.visit_expr(*value);
        }
        if let Some(value) = node.format_spec {
            self.visit_expr(*value);
        }
    }
    fn visit_expr_joined_str(&mut self, node: ExprJoinedStr<R>) {
        self.generic_visit_expr_joined_str(node)
    }
    fn generic_visit_expr_joined_str(&mut self, node: ExprJoinedStr<R>) {
        for value in node.values {
            self.visit_expr(value);
        }
    }
    fn visit_expr_constant(&mut self, node: ExprConstant<R>) {
        self.generic_visit_expr_constant(node)
    }
    fn generic_visit_expr_constant(&mut self, node: ExprConstant<R>) {}
    fn visit_expr_attribute(&mut self, node: ExprAttribute<R>) {
        self.generic_visit_expr_attribute(node)
    }
    fn generic_visit_expr_attribute(&mut self, node: ExprAttribute<R>) {
        {
            let value = node.value;
            self.visit_expr(*value);
        }
    }
    fn visit_expr_subscript(&mut self, node: ExprSubscript<R>) {
        self.generic_visit_expr_subscript(node)
    }
    fn generic_visit_expr_subscript(&mut self, node: ExprSubscript<R>) {
        {
            let value = node.value;
            self.visit_expr(*value);
        }
        {
            let value = node.slice;
            self.visit_expr(*value);
        }
    }
    fn visit_expr_starred(&mut self, node: ExprStarred<R>) {
        self.generic_visit_expr_starred(node)
    }
    fn generic_visit_expr_starred(&mut self, node: ExprStarred<R>) {
        {
            let value = node.value;
            self.visit_expr(*value);
        }
    }
    fn visit_expr_name(&mut self, node: ExprName<R>) {
        self.generic_visit_expr_name(node)
    }
    fn generic_visit_expr_name(&mut self, node: ExprName<R>) {}
    fn visit_expr_list(&mut self, node: ExprList<R>) {
        self.generic_visit_expr_list(node)
    }
    fn generic_visit_expr_list(&mut self, node: ExprList<R>) {
        for value in node.elts {
            self.visit_expr(value);
        }
    }
    fn visit_expr_tuple(&mut self, node: ExprTuple<R>) {
        self.generic_visit_expr_tuple(node)
    }
    fn generic_visit_expr_tuple(&mut self, node: ExprTuple<R>) {
        for value in node.elts {
            self.visit_expr(value);
        }
    }
    fn visit_expr_slice(&mut self, node: ExprSlice<R>) {
        self.generic_visit_expr_slice(node)
    }
    fn generic_visit_expr_slice(&mut self, node: ExprSlice<R>) {
        if let Some(value) = node.lower {
            self.visit_expr(*value);
        }
        if let Some(value) = node.upper {
            self.visit_expr(*value);
        }
        if let Some(value) = node.step {
            self.visit_expr(*value);
        }
    }
    fn visit_expr_context(&mut self, node: ExprContext) {
        self.generic_visit_expr_context(node)
    }
    fn generic_visit_expr_context(&mut self, node: ExprContext) {}
    fn visit_boolop(&mut self, node: BoolOp) {
        self.generic_visit_boolop(node)
    }
    fn generic_visit_boolop(&mut self, node: BoolOp) {}
    fn visit_operator(&mut self, node: Operator) {
        self.generic_visit_operator(node)
    }
    fn generic_visit_operator(&mut self, node: Operator) {}
    fn visit_unaryop(&mut self, node: UnaryOp) {
        self.generic_visit_unaryop(node)
    }
    fn generic_visit_unaryop(&mut self, node: UnaryOp) {}
    fn visit_cmpop(&mut self, node: CmpOp) {
        self.generic_visit_cmpop(node)
    }
    fn generic_visit_cmpop(&mut self, node: CmpOp) {}
    fn visit_comprehension(&mut self, node: Comprehension<R>) {
        self.generic_visit_comprehension(node)
    }
    fn generic_visit_comprehension(&mut self, node: Comprehension<R>) {}
    fn visit_excepthandler(&mut self, node: ExceptHandler<R>) {
        self.generic_visit_excepthandler(node)
    }
    fn generic_visit_excepthandler(&mut self, node: ExceptHandler<R>) {
        match node {
            ExceptHandler::ExceptHandler(data) => self.visit_excepthandler_except_handler(data),
        }
    }
    fn visit_excepthandler_except_handler(&mut self, node: ExceptHandlerExceptHandler<R>) {
        self.generic_visit_excepthandler_except_handler(node)
    }
    fn generic_visit_excepthandler_except_handler(&mut self, node: ExceptHandlerExceptHandler<R>) {
        if let Some(value) = node.type_ {
            self.visit_expr(*value);
        }
        for value in node.body {
            self.visit_stmt(value);
        }
    }
    fn visit_arguments(&mut self, node: Arguments<R>) {
        self.generic_visit_arguments(node)
    }
    fn generic_visit_arguments(&mut self, node: Arguments<R>) {}
    fn visit_arg(&mut self, node: Arg<R>) {
        self.generic_visit_arg(node)
    }
    fn generic_visit_arg(&mut self, node: Arg<R>) {}
    fn visit_keyword(&mut self, node: Keyword<R>) {
        self.generic_visit_keyword(node)
    }
    fn generic_visit_keyword(&mut self, node: Keyword<R>) {}
    fn visit_alias(&mut self, node: Alias<R>) {
        self.generic_visit_alias(node)
    }
    fn generic_visit_alias(&mut self, node: Alias<R>) {}
    fn visit_withitem(&mut self, node: WithItem<R>) {
        self.generic_visit_withitem(node)
    }
    fn generic_visit_withitem(&mut self, node: WithItem<R>) {}
    fn visit_match_case(&mut self, node: MatchCase<R>) {
        self.generic_visit_match_case(node)
    }
    fn generic_visit_match_case(&mut self, node: MatchCase<R>) {}
    fn visit_pattern(&mut self, node: Pattern<R>) {
        self.generic_visit_pattern(node)
    }
    fn generic_visit_pattern(&mut self, node: Pattern<R>) {
        match node {
            Pattern::MatchValue(data) => self.visit_pattern_match_value(data),
            Pattern::MatchSingleton(data) => self.visit_pattern_match_singleton(data),
            Pattern::MatchSequence(data) => self.visit_pattern_match_sequence(data),
            Pattern::MatchMapping(data) => self.visit_pattern_match_mapping(data),
            Pattern::MatchClass(data) => self.visit_pattern_match_class(data),
            Pattern::MatchStar(data) => self.visit_pattern_match_star(data),
            Pattern::MatchAs(data) => self.visit_pattern_match_as(data),
            Pattern::MatchOr(data) => self.visit_pattern_match_or(data),
        }
    }
    fn visit_pattern_match_value(&mut self, node: PatternMatchValue<R>) {
        self.generic_visit_pattern_match_value(node)
    }
    fn generic_visit_pattern_match_value(&mut self, node: PatternMatchValue<R>) {
        {
            let value = node.value;
            self.visit_expr(*value);
        }
    }
    fn visit_pattern_match_singleton(&mut self, node: PatternMatchSingleton<R>) {
        self.generic_visit_pattern_match_singleton(node)
    }
    fn generic_visit_pattern_match_singleton(&mut self, node: PatternMatchSingleton<R>) {}
    fn visit_pattern_match_sequence(&mut self, node: PatternMatchSequence<R>) {
        self.generic_visit_pattern_match_sequence(node)
    }
    fn generic_visit_pattern_match_sequence(&mut self, node: PatternMatchSequence<R>) {
        for value in node.patterns {
            self.visit_pattern(value);
        }
    }
    fn visit_pattern_match_mapping(&mut self, node: PatternMatchMapping<R>) {
        self.generic_visit_pattern_match_mapping(node)
    }
    fn generic_visit_pattern_match_mapping(&mut self, node: PatternMatchMapping<R>) {
        for value in node.keys {
            self.visit_expr(value);
        }
        for value in node.patterns {
            self.visit_pattern(value);
        }
    }
    fn visit_pattern_match_class(&mut self, node: PatternMatchClass<R>) {
        self.generic_visit_pattern_match_class(node)
    }
    fn generic_visit_pattern_match_class(&mut self, node: PatternMatchClass<R>) {
        {
            let value = node.cls;
            self.visit_expr(*value);
        }
        for value in node.patterns {
            self.visit_pattern(value);
        }
        for value in node.kwd_patterns {
            self.visit_pattern(value);
        }
    }
    fn visit_pattern_match_star(&mut self, node: PatternMatchStar<R>) {
        self.generic_visit_pattern_match_star(node)
    }
    fn generic_visit_pattern_match_star(&mut self, node: PatternMatchStar<R>) {}
    fn visit_pattern_match_as(&mut self, node: PatternMatchAs<R>) {
        self.generic_visit_pattern_match_as(node)
    }
    fn generic_visit_pattern_match_as(&mut self, node: PatternMatchAs<R>) {
        if let Some(value) = node.pattern {
            self.visit_pattern(*value);
        }
    }
    fn visit_pattern_match_or(&mut self, node: PatternMatchOr<R>) {
        self.generic_visit_pattern_match_or(node)
    }
    fn generic_visit_pattern_match_or(&mut self, node: PatternMatchOr<R>) {
        for value in node.patterns {
            self.visit_pattern(value);
        }
    }
    fn visit_type_param(&mut self, node: TypeParam<R>) {
        self.generic_visit_type_param(node)
    }
    fn generic_visit_type_param(&mut self, node: TypeParam<R>) {
        match node {
            TypeParam::TypeVar(data) => self.visit_type_param_type_var(data),
            TypeParam::ParamSpec(data) => self.visit_type_param_param_spec(data),
            TypeParam::TypeVarTuple(data) => self.visit_type_param_type_var_tuple(data),
        }
    }
    fn visit_type_param_type_var(&mut self, node: TypeParamTypeVar<R>) {
        self.generic_visit_type_param_type_var(node)
    }
    fn generic_visit_type_param_type_var(&mut self, node: TypeParamTypeVar<R>) {
        if let Some(value) = node.bound {
            self.visit_expr(*value);
        }
    }
    fn visit_type_param_param_spec(&mut self, node: TypeParamParamSpec<R>) {
        self.generic_visit_type_param_param_spec(node)
    }
    fn generic_visit_type_param_param_spec(&mut self, node: TypeParamParamSpec<R>) {}
    fn visit_type_param_type_var_tuple(&mut self, node: TypeParamTypeVarTuple<R>) {
        self.generic_visit_type_param_type_var_tuple(node)
    }
    fn generic_visit_type_param_type_var_tuple(&mut self, node: TypeParamTypeVarTuple<R>) {}
}
