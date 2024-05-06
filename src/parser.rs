use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar);

pub type ExprParser = grammar::ExprParser;
pub type ScriptParser = grammar::ScriptParser;
