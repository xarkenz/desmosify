#![allow(dead_code)]

use desmosify;

fn main() {
    let code = include_str!("stratego.desmos");
    let tokens = match desmosify::token::tokenize(code) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("DesmosifyError while lexing: {error}");
            return;
        }
    };
    for token in &tokens {
        print!("{} ", &code[token.start.index .. token.end.index]);
    }
    println!();
    println!();
    let definitions = match desmosify::syntax::parse_definitions(&tokens) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("DesmosifyError while parsing: {error}");
            return;
        }
    };
    println!("{definitions:?}");
}