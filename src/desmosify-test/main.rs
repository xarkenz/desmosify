#![allow(dead_code)]

use desmosify;

fn main() {
    let code = include_str!("goal.txt");
    let tokens = desmosify::lex_tokens_from_str(code).unwrap();
    for token in &tokens {
        print!("{} ", &code[token.start.index..token.end.index]);
    }
    println!();
    println!();
    let definitions = match desmosify::parse_definitions(&tokens) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("DesmosifyError: {error}");
            return;
        }
    };
    println!("{definitions:?}");
}