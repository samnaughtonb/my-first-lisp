use std::collections::HashMap;

use crate::ast;

#[derive(Debug)]
struct Env<'a> {
    data: HashMap<ast::Symbol, ast::Expr>,
    outer: Option<&'a Env<'a>>,
}
