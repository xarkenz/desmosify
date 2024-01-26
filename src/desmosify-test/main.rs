use desmosify::{self, target::Target};

fn main() {
    let code = "
        public {
            \"fibonacci!\";
            action next();
            \"the numbers:\";
            num_a;
            num_b;
        }

        var num_a: int = 0;
        var num_b: int = 1;

        action next() {
            num_a := num_b,
            num_b := num_a + num_b,
        }
    ";

    let tokens = match desmosify::token::tokenize(code) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("DesmosifyError while tokenizing: {error}");
            return;
        }
    };
    // for token in &tokens {
    //     print!("{} ", &code[token.start.index .. token.end.index]);
    // }
    // println!();
    // println!();
    let (signatures, mut definitions) = match desmosify::syntax::parse(&tokens) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("DesmosifyError while parsing: {error}");
            return;
        }
    };
    println!("{signatures:?}");
    if let Err(error) = desmosify::semantics::analyze(&signatures, &mut definitions) {
        eprintln!("DesmosifyError while analyzing: {error}");
        return;
    }
    println!();
    let target = desmosify::target::desmos::GeometryTarget;
    let compiled = target.compile(&definitions, &signatures);
    println!("{compiled}");
    println!("done");
}