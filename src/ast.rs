use std::fmt::{Display, Error, Formatter};

pub type Symbol = String;

#[derive(Clone)]
pub struct Script(pub Vec<Expr>);

#[derive(Clone)]
pub enum Expr {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Symbol(Symbol),
    List(Vec<Expr>),
}

impl Display for Script {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        for expr in self.0.iter() {
            write!(fmt, "{}", expr)?;
        }
        Ok(())
    }
}

impl Display for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Expr::Bool(b) => write!(fmt, "{}", b),
            Expr::Integer(i) => write!(fmt, "{}", i),
            Expr::Float(f) => write!(fmt, "{}", f),
            Expr::Symbol(sym) => write!(fmt, "{}", sym),
            Expr::List(list) => {
                write!(fmt, "(")?;
                for item in list.iter() {
                    write!(fmt, "{} ", item)?;
                }
                write!(fmt, ")")
            },
        }
    }
}
