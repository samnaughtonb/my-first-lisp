use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::rc::Rc;

use crate::ast;

pub enum Value<'a> {
    Bool(bool),
    Integer(i64),
    Float(f64),
    List(Vec<Value<'a>>),
    Func(Func<'a>),
}

pub enum Func<'a> {
    BuiltIn {
        name: &'a str,
        func: fn(&mut Env<'a>, &[ast::Expr]) -> Result<Rc<Value<'a>>, String>,
    },
    UserDefined {
        n_args: usize,
        body: ast::Expr,
    }
}

pub struct Env<'a> {
    data: HashMap<ast::Symbol, Rc<Value<'a>>>,
    outer: Option<&'a Env<'a>>,
}

macro_rules! insert_builtin {
    ($data:ident, $name:expr, $func:ident) => {
        $data.insert(
            ast::Symbol::from($name),
            Rc::new(Value::Func(
                Func::BuiltIn {
                    name: stringify!($func),
                    func: $func,
                }
            ))
        );
    };
}

impl<'a> Env<'a> {

    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            outer: None,
        }
    }

    pub fn default() -> Self {
        let mut env = Self::new();
        insert_builtin!(env, "+", add);
        env
    }

    fn insert(&mut self, key: ast::Symbol, value: Rc<Value<'a>>) {
        self.data.insert(key, value);
    }

    fn get(&self, key: &ast::Symbol) -> Option<Rc<Value<'a>>> {
        match self.data.get(key) {
            Some(val) => Some(val.clone()),
            None => self.outer.and_then(|env| env.get(key)),
        }
    }

    pub fn eval(&mut self, expr: &ast::Expr) -> Result<Rc<Value<'a>>, String> {
        match expr {
            ast::Expr::Bool(b) => Ok(Rc::new(Value::Bool(b.clone()))),
            ast::Expr::Integer(i) => Ok(Rc::new(Value::Integer(i.clone()))),
            ast::Expr::Float(f) => Ok(Rc::new(Value::Float(f.clone()))),
            ast::Expr::Symbol(sym) => match self.get(sym) {
                Some(val) => Ok(val.clone()),
                None => { Err(format!("Unknown symbol '{}'", sym)) },
            },
            ast::Expr::List(list) => {
                let (first, rest) = list.split_first().ok_or("List cannot be empty")?;
                let res = self.eval(&first)?;
                match res.as_ref() {
                    Value::Func(f) => match f {
                        Func::BuiltIn { func, .. } => {
                            let value = (*func)(self, &rest[..])?;
                            Ok(value.clone())
                        },
                        Func::UserDefined { n_args, body } => {
                            Ok(Rc::new(Value::Integer(0)))
                        }
                    },
                    _ => Err(format!("{} is not a function", first)),
                }
            }
        }
    }
}

fn add<'a>(env: &mut Env<'a>, args: &[ast::Expr]) -> Result<Rc<Value<'a>>, String> {
    let (first, rest) = args.split_first().ok_or("Cannot apply '+' to zero arguments")?;
    let first = env.eval(&first)?;
    match first.as_ref() {
        Value::Integer(i) => {
            let mut sum: i64 = *i;
            for item in rest {
                let value = env.eval(&item)?;
                let value = match value.as_ref() {
                    Value::Integer(j) => Ok(j),
                    _ => Err(format!("Non-integer '{}' found in integer sum", value))
                }?;
                sum += value;
            }
            Ok(Rc::new(Value::Integer(sum)))
        },
        Value::Float(f) => {
            let mut sum: f64 = *f;
            for item in rest {
                let value = env.eval(&item)?;
                let value = match value.as_ref() {
                    Value::Float(g) => Ok(g),
                    _ => Err(format!("Non-float '{}' found in float sum", value))
                }?;
                sum += value;
            }
            Ok(Rc::new(Value::Float(sum)))
        },
        _ => Err("Must apply '+' to numeric types".to_string())
    }
}

impl Display for Value<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Value::Bool(b) => write!(fmt, "{}", b),
            Value::Integer(i) => write!(fmt, "{}", i),
            Value::Float(f) => write!(fmt, "{}", f),
            Value::List(list) => {
                let _ = write!(fmt, "(")?;
                for item in list.iter() {
                    let _ = write!(fmt, "{} ", item)?;
                }
                write!(fmt, ")")
            },
            Value::Func(func) => match func {
                Func::BuiltIn { name, .. } =>
                    write!(fmt, "<built-in function '{}'>", name),
                Func::UserDefined { .. } =>
                    write!(fmt, "function")
            }
        }
    }
}

impl Display for Func<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}", "func")
    }
}

