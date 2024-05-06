pub type Symbol = String;

#[derive(Debug)]
pub struct Script(pub Vec<Expr>);

#[derive(Debug)]
pub enum Expr {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Symbol(Symbol),
    List(Vec<Expr>),
}
