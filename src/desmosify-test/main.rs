use std::time::Instant;

fn main() {
    let start_time = Instant::now();

    let args = desmosify::cli::parse_command_line_args();
    match desmosify::cli::invoke(&args) {
        Err(error) => println!("\x1b[31mError: {error}\x1b[0m"),
        Ok(_) => println!("\x1b[32mFinished\x1b[0m"),
    }

    let time_taken_ms = start_time.elapsed().as_millis();
    println!("\x1b[2mTime: {time_taken_ms} ms\x1b[22m");
}