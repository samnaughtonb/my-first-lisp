use std::io;
use std::io::Write;

use clap::{Parser, Subcommand};

pub mod ast;
pub mod eval;
pub mod parser;


#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        path: String,
    },
    Repl,
}


fn main() {

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { path: _ } => {
            unimplemented!();
        },
        Commands::Repl => {
            let inst = parser::ExprParser::new();
            let mut env = eval::Env::default();
            loop {
                print!("sam's lisp >> ");
                io::stdout().flush().unwrap();

                let mut script = String::new();
                let _ = io::stdin().read_line(&mut script);
                match inst.parse(&script) {
                    Ok(tree) => {
                        let tree_cloned = tree.clone();
                        match env.eval(&tree_cloned) {
                            Ok(res) => println!("🔥 {}", res),
                            Err(msg) => {
                                println!("😱 ERROR: {}", msg);
                                if cli.debug { println!("   TREE:  {}", tree); }
                            }
                        }
                    },
                    Err(msg) => println!("\n😱 PARSER ERROR: {}", msg),
                }
            }
        },
    }
}
