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
    Compile {
        path: String,
    },
    Run {
        path: String,
    },
    Repl,
}


fn main() {

    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { path: _ } => {
            unimplemented!();
        },
        Commands::Run { path: _ } => {
            unimplemented!();
        },
        Commands::Repl => {
            let inst = parser::ScriptParser::new();
            loop {
                print!("sam's lisp >> ");
                io::stdout().flush().unwrap();

                let mut script = String::new();
                let _ = io::stdin().read_line(&mut script);                
                let res = inst.parse(&script);

                if cli.debug {
                    match res {
                        Ok(res) => println!("🔥 {:?}", res),
                        Err(err) => println!("😱 {:?}", err),
                    }
                }
            }
        },
    }
}
