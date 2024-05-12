use std::borrow::BorrowMut;
use std::cell::RefCell;
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
        params: Vec<ast::Expr>,
        body: ast::Expr,
    }
}

#[derive(Clone)]
pub struct Env<'a> {
    data: HashMap<ast::Symbol, Rc<Value<'a>>>,
    outer: Option<Rc<RefCell<Env<'a>>>>, // &'a Env<'a>>,
}

macro_rules! insert_builtin {
    ($data:ident, $name:expr, $func:ident) => {
        insert_builtin!($data, $name, $func, stringify!($func));
    };
    ($data:ident, $name:expr, $func:ident, $tag:expr) => {
        $data.insert(
            ast::Symbol::from($name),
            Rc::new(
                Value::Func(
                    Func::BuiltIn {
                        name: $tag,
                        func: $func,
                    }
                )
            )
        );
    }
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
        insert_builtin!(env, "def", def);
        insert_builtin!(env, "fn", func, "fn");
        insert_builtin!(env, "if", ifdef, "if");
        insert_builtin!(env, "=", equals);
        insert_builtin!(env, "+", addition);
        insert_builtin!(env, "-", subtraction);
        insert_builtin!(env, "*", multiplication);
        insert_builtin!(env, "/", division);
        insert_builtin!(env, "<", less_than);
        env
    }

    fn insert(&mut self, key: ast::Symbol, value: Rc<Value<'a>>) {
        self.data.insert(key, value);
    }

    fn get(&self, key: &ast::Symbol) -> Option<Rc<Value<'a>>> {
        match self.data.get(key) {
            Some(val) => Some(Rc::clone(val)),
            None => self.outer.as_ref()?.borrow().get(key),
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
                        Func::UserDefined { params, body } => {
                            if params.len() != rest.len() {
                                return Err("Incorrect number of arguments provided".to_string());
                            }
                            let mut new_env = Env {
                                data: HashMap::new(),
                                outer: Some(Rc::new(RefCell::new(self.clone()))),
                            };
                            for (param, arg) in params.iter().zip(rest.iter()) {
                                let _ = match param {
                                    ast::Expr::Symbol(sym) => {
                                        let arg_val = self.borrow_mut().eval(&arg)?;
                                        new_env.borrow_mut().insert(ast::Symbol::from(sym), arg_val);
                                        Ok(())
                                    },
                                    _ => Err("..."),
                                }?;
                            }
                            new_env.borrow_mut().eval(&body)
                        }
                    },
                    _ => Err(format!("{} is not a function", first)),
                }
            }
        }
    }
}

fn func<'a>(env: &mut Env<'a>, args: &[ast::Expr]) -> Result<Rc<Value<'a>>, String> {
    if args.len() != 2 {
        return Err("'fn' takes 2 arguments only".to_string());
    }
    let params = match args.get(0).unwrap() {
        ast::Expr::List(list) => Ok(list),
        _ => Err("..."),
    }?;
    let body = args.get(1).unwrap();
    Ok(Rc::new(Value::Func(Func::UserDefined { 
        params: params.to_vec(),
        body: body.clone() })))
}

fn ifdef<'a>(env: &mut Env<'a>, args: &[ast::Expr]) -> Result<Rc<Value<'a>>, String> {
    if args.len() != 3 {
        return Err("'if' takes 3 arguments".to_string());
    }
    let cond = env.eval(args.get(0).unwrap())?;
    match cond.as_ref() {
        Value::Bool(b) => match b {
            true => env.eval(args.get(1).unwrap()),
            false => env.eval(args.get(2).unwrap()),
        },
        _ => Err("Condition in 'if' must evaluate to a boolean value".to_string())
    }
}

fn def<'a>(env: &mut Env<'a>, args: &[ast::Expr]) -> Result<Rc<Value<'a>>, String> {
    if args.len() != 2 {
        return Err("'def' takes 2 arguments only".to_string());
    }
    let name = match args.get(0) {
        Some(ast::Expr::Symbol(sym)) => Ok(sym),
        _ => Err("First argument to 'def' must be a symbol"),
    }?;
    let value = env.eval(args.get(1).unwrap())?;
    let sym = ast::Symbol::from(name);
    env.insert(sym, value);
    Ok(Rc::new(Value::Integer(0)))
}

macro_rules! arithmetic_builtin {
    ($name:ident, $op:tt) => {
        fn $name<'a>(env: &mut Env<'a>, args: &[ast::Expr]) -> Result<Rc<Value<'a>>, String> {
            let (first, rest) = args.split_first().ok_or(concat!("Cannot apply '", stringify!($op), "' to zero arguments"))?;
            let first = env.eval(&first)?;
            match first.as_ref() {
                Value::Integer(i) => {
                    let mut res: i64 = *i;
                    for item in rest {
                        let value = env.eval(&item)?;
                        let value = match value.as_ref() {
                            Value::Integer(j) => Ok(j),
                            _ => Err(format!(concat!("Non-integer '{}' found in integer ", stringify!($name)), value))
                        }?;
                        res = res $op value;
                    }
                    Ok(Rc::new(Value::Integer(res)))
                },
                Value::Float(f) => {
                    let mut res: f64 = *f;
                    for item in rest {
                        let value = env.eval(&item)?;
                        let value = match value.as_ref() {
                            Value::Float(g) => Ok(g),
                            _ => Err(format!(concat!("Non-float '{}' found in float ", stringify!($name)), value))
                        }?;
                        res = res $op value;
                    }
                    Ok(Rc::new(Value::Float(res)))
                },
                _ => Err("Must apply '+' to numeric types".to_string())
            }
        }
    };
}

arithmetic_builtin!(addition, +);
arithmetic_builtin!(subtraction, -);
arithmetic_builtin!(multiplication, *);
arithmetic_builtin!(division, /);

fn equals<'a>(env: &mut Env<'a>, args: &[ast::Expr]) -> Result<Rc<Value<'a>>, String> {
    if args.len() < 2 {
        return Err("Cannot apply '=' to fewer than 2 arguments".to_string());
    }
    let (first, rest) = args.split_first().unwrap();
    let first = env.eval(&first)?;
    match first.as_ref() {
        Value::Bool(b) => {
            let mut res = true;
            for item in rest {
                let value = env.eval(&item)?;
                res = match value.as_ref() {
                    Value::Bool(c) => Ok(res && (b == c)),
                    _ => Err(format!("Non-boolean '{}' found in boolean equals", value)),
                }?;
                if !res { break; }
            }
            Ok(Rc::new(Value::Bool(res)))
        },
        Value::Integer(i) => {
            let mut res = true;
            for item in rest {
                let value = env.eval(&item)?;
                res = match value.as_ref() {
                    Value::Integer(j) => Ok(res && (i == j)),
                    _ => Err(format!("Non-integer '{}' found in integer equals", value)),
                }?;
                if !res { break; }
            }
            Ok(Rc::new(Value::Bool(res)))
        },
        Value::Float(f) => {
            let mut res = true;
            for item in rest {
                let value = env.eval(&item)?;
                res = match value.as_ref() {
                    Value::Float(g) => Ok(res && (f == g)),
                    _ => Err(format!("Non-float '{}' found in float equals", value)),
                }?;
                if !res { break; }
            }
            Ok(Rc::new(Value::Bool(res)))
        },
        _ => Err("Must apply '=' to numeric or boolean types".to_string()),
    }
}

fn less_than<'a>(env: &mut Env<'a>, args: &[ast::Expr]) -> Result<Rc<Value<'a>>, String> {
    if args.len() < 2 {
        return Err("Cannot apply '<' to fewer than 2 arguments".to_string());
    }
    let (first, rest) = args.split_first().unwrap();
    let first = env.eval(&first)?;
    match first.as_ref() {
        Value::Integer(i) => {
            let second = args.get(1).unwrap();
            let second = env.eval(&second)?;
            match second.as_ref() {
                Value::Integer(j) => Ok(Rc::new(Value::Bool(i < j))),
                _ => Err(format!("Cannot compare integer '{}' with non-integer '{}'", first, second)),
            }
        },
        Value::Float(f) => {
            let second = args.get(1).unwrap();
            let second = env.eval(&second)?;
            match second.as_ref() {
                Value::Float(g) => Ok(Rc::new(Value::Bool(f < g))),
                _ => Err(format!("Cannot compare float '{}' with non-float '{}'", first, second)),
            }
        },
        _ => Err(format!("Cannot use value '{}' for comparison", first)),
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

