use std::time::Instant;
use desmosify::cli::DesmosifyArgs;

fn main() {
    let start_time = Instant::now();

    // let args = desmosify::cli::parse_command_line_args();
    let args = DesmosifyArgs {
        src: vec!["src/desmosify-test/stratego.desmos".into()],
        out: "src/desmosify-test/stratego-out.json".into(),
        debug: false,
    };
    match desmosify::cli::invoke(&args) {
        Err(error) => println!("\x1b[31m{}\x1b[0m", error.to_string_with_context(args.source_paths())),
        Ok(_) => println!("\x1b[32mFinished\x1b[0m"),
    }

    let time_taken_ms = start_time.elapsed().as_millis();
    println!("\x1b[2mTime: {time_taken_ms} ms\x1b[22m");
}
