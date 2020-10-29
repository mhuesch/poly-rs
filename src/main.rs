use combine::parser::Parser;
use combine::stream::easy;
use rustyline::{error::ReadlineError, Editor};

use poly::{env::*, infer::*, parse::expr};

fn main() {
    println!("hello, poly & Rust!");

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match expr().parse(easy::Stream(&line[..])) {
                    Err(err) => println!("parse error: {}", err),
                    Ok((e, _)) => {
                        println!("ast: {:?}\n", e);
                        let env = Env::new();
                        match constraints_expr(env, e) {
                            Err(err) => println!("type error: {:?}", err),
                            Ok((csts, subst, ty, sc)) => {
                                println!("constraints: {:?}\n", csts);
                                println!("subst: {:?}\n", subst);
                                println!("type: {:?}\n", ty);
                                println!("scheme: {:?}", sc);
                            }
                        }
                    }
                };
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => {
                println!("\nbye!");
                break;
            }
            Err(err) => {
                println!("error: {:?}", err);
                break;
            }
        }
    }
}
